use anyhow::{Context as _, Result, bail};
use clash_verge_logging::{Type, logging};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::utils::dirs::APP_ID;

const CURRENT_APP_BUNDLE: &str = "Clash Verge.app";
const LEGACY_APP_BUNDLE: &str = "Clash Verge for v2free.app";
const MAIN_BINARY: &str = "clash-verge";

/// Remove the legacy fork `.app` when running from the unified bundle name.
pub fn remove_legacy_bundle_if_needed() {
    if let Err(err) = try_remove_legacy_bundle() {
        logging!(
            warn,
            Type::Setup,
            "Legacy macOS app bundle cleanup skipped: {err}"
        );
    }
}

fn try_remove_legacy_bundle() -> Result<()> {
    let exe = tauri::utils::platform::current_exe().context("current_exe")?;
    let current_bundle = app_bundle_path(&exe).context("resolve current .app bundle")?;

    if current_bundle.file_name() != Some(OsStr::new(CURRENT_APP_BUNDLE)) {
        return Ok(());
    }

    let current_canonical = dunce::canonicalize(&current_bundle).unwrap_or(current_bundle);

    for legacy_path in legacy_bundle_candidates() {
        if !legacy_path.is_dir() {
            continue;
        }

        if !is_legacy_bundle(&legacy_path)? {
            continue;
        }

        let legacy_canonical = dunce::canonicalize(&legacy_path).unwrap_or(legacy_path.clone());
        if legacy_canonical == current_canonical {
            continue;
        }

        trash_bundle(&legacy_path)?;
        logging!(
            info,
            Type::Setup,
            "Removed legacy app bundle: {}",
            legacy_path.display()
        );
    }

    Ok(())
}

fn legacy_bundle_candidates() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("/Applications").join(LEGACY_APP_BUNDLE)];

    if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join("Applications").join(LEGACY_APP_BUNDLE));
    }

    paths
}

fn app_bundle_path(exe: &Path) -> Option<PathBuf> {
    let mut path = exe.to_path_buf();
    while path.pop() {
        if path.extension() == Some(OsStr::new("app")) {
            return Some(path);
        }
    }
    None
}

fn is_legacy_bundle(bundle: &Path) -> Result<bool> {
    if bundle.file_name() != Some(OsStr::new(LEGACY_APP_BUNDLE)) {
        return Ok(false);
    }

    let binary = bundle.join("Contents/MacOS").join(MAIN_BINARY);
    if !binary.is_file() {
        return Ok(false);
    }

    let plist = bundle.join("Contents/Info.plist");
    if !plist.is_file() {
        return Ok(false);
    }

    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleIdentifier", "raw", "-o", "-"])
        .arg(&plist)
        .output()
        .context("read CFBundleIdentifier")?;

    if !output.status.success() {
        return Ok(false);
    }

    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(bundle_id == APP_ID)
}

fn trash_bundle(path: &Path) -> Result<()> {
    let absolute = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let path_str = absolute.to_string_lossy().replace('"', "\\\"");

    let script = format!(r#"tell application "Finder" to delete POSIX file "{path_str}""#);
    let output = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .output()
        .context("osascript trash")?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Finder trash failed: {stderr}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_bundle_path_finds_parent_app() {
        let exe = Path::new("/Applications/Clash Verge.app/Contents/MacOS/clash-verge");
        let bundle = app_bundle_path(exe).expect("bundle");
        assert_eq!(bundle, Path::new("/Applications/Clash Verge.app"));
    }

    #[test]
    fn app_bundle_path_returns_none_outside_app() {
        assert!(app_bundle_path(Path::new("/usr/bin/clash-verge")).is_none());
    }
}
