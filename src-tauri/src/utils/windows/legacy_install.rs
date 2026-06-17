use anyhow::{Context as _, Result};
use clash_verge_logging::{Type, logging};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

const CURRENT_DIR_NAME: &str = "Clash Verge";
const LEGACY_DIR_NAME: &str = "Clash Verge for v2free";
const MAIN_BINARY: &str = "clash-verge.exe";

/// Migrate off the legacy install folder on Windows.
pub fn remove_legacy_install_if_needed() {
    if let Err(err) = try_remove_legacy_install() {
        logging!(
            warn,
            Type::Setup,
            "Legacy Windows install folder cleanup skipped: {err}"
        );
    }
}

fn try_remove_legacy_install() -> Result<()> {
    let exe = tauri::utils::platform::current_exe().context("current_exe")?;
    let current_dir = install_dir(&exe).context("resolve install dir")?;

    if is_legacy_install_dir(&current_dir) {
        return relaunch_from_unified_if_present(&exe);
    }

    if !is_unified_install_dir(&current_dir) {
        return Ok(());
    }

    for legacy_dir in legacy_install_candidates() {
        if !should_remove_legacy(&legacy_dir, &current_dir) {
            continue;
        }

        remove_legacy_tree(&legacy_dir)?;
        cleanup_legacy_registry();
        logging!(
            info,
            Type::Setup,
            "Removed legacy install folder: {}",
            legacy_dir.display()
        );
    }

    Ok(())
}

fn relaunch_from_unified_if_present(current_exe: &Path) -> Result<()> {
    for unified_dir in unified_install_candidates() {
        let unified_exe = unified_dir.join(MAIN_BINARY);
        if !unified_exe.is_file() {
            continue;
        }
        if paths_equal(&unified_exe, current_exe) {
            continue;
        }

        logging!(
            info,
            Type::Setup,
            "Relaunching from unified install: {}",
            unified_exe.display()
        );
        StdCommand::new(&unified_exe).spawn().context("relaunch unified")?;
        std::process::exit(0);
    }

    Ok(())
}

fn should_remove_legacy(legacy_dir: &Path, current_dir: &Path) -> bool {
    if !legacy_dir.is_dir() {
        return false;
    }
    if paths_equal(legacy_dir, current_dir) {
        return false;
    }
    legacy_dir.join(MAIN_BINARY).is_file()
}

fn install_dir(exe: &Path) -> Option<PathBuf> {
    exe.parent().map(Path::to_path_buf)
}

fn is_unified_install_dir(dir: &Path) -> bool {
    dir.file_name().is_some_and(|name| name == CURRENT_DIR_NAME)
}

fn is_legacy_install_dir(dir: &Path) -> bool {
    dir.file_name().is_some_and(|name| name == LEGACY_DIR_NAME)
}

fn program_files_dirs() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(pf) = std::env::var("ProgramFiles") {
        paths.push(PathBuf::from(pf));
    }
    if let Ok(pf64) = std::env::var("ProgramFiles(x86)") {
        paths.push(PathBuf::from(pf64));
    }
    if paths.is_empty() {
        paths.push(PathBuf::from(r"C:\Program Files"));
        paths.push(PathBuf::from(r"C:\Program Files (x86)"));
    }
    paths.sort();
    paths.dedup();
    paths
}

fn legacy_install_candidates() -> Vec<PathBuf> {
    program_files_dirs()
        .into_iter()
        .map(|root| root.join(LEGACY_DIR_NAME))
        .collect()
}

fn unified_install_candidates() -> Vec<PathBuf> {
    program_files_dirs()
        .into_iter()
        .map(|root| root.join(CURRENT_DIR_NAME))
        .collect()
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    let a = dunce::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let b = dunce::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    a == b
}

fn remove_legacy_tree(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().into_owned();
    let args = ["/C", "rmdir", "/S", "/Q", &path_str];

    if try_rmdir(StdCommand::new("cmd").args(args))? {
        return Ok(());
    }

    #[cfg(windows)]
    {
        use deelevate::{PrivilegeLevel, Token};
        use runas::Command as RunasCommand;

        let token = Token::with_current_process().context("current process token")?;
        if token.privilege_level()? == PrivilegeLevel::NotPrivileged {
            let status = RunasCommand::new("cmd")
                .args(&args)
                .show(false)
                .status()
                .context("elevated rmdir")?;
            if status.success() {
                return Ok(());
            }
        }
    }

    std::fs::remove_dir_all(path).context("remove legacy install dir")
}

fn try_rmdir(command: &mut StdCommand) -> Result<bool> {
    match command.status() {
        Ok(status) if status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
        Err(err) => Err(err.into()),
    }
}

#[cfg(windows)]
fn cleanup_legacy_registry() {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let paths = [
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Clash Verge for v2free",
        r"Software\v2free\Clash Verge for v2free",
    ];

    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let root = RegKey::predef(hive);
        for path in paths {
            let _ = root.delete_subkey_all(path);
        }
    }
}

#[cfg(not(windows))]
fn cleanup_legacy_registry() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_install_dir_matches_product_folder() {
        let dir = Path::new(r"C:\Program Files\Clash Verge");
        assert!(is_unified_install_dir(dir));
    }

    #[test]
    fn legacy_install_dir_is_detected() {
        let dir = Path::new(r"C:\Program Files\Clash Verge for v2free");
        assert!(is_legacy_install_dir(dir));
    }
}
