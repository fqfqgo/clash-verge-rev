use super::CmdResult;
use crate::config::Config;
use crate::core::{CoreManager, autostart, handle};
use crate::{cmd::StringifyErr as _, feat, utils::{self, dirs}};
use smartstring::alias::String;
use std::process::Command;
use tauri::{AppHandle, Manager as _};
use tokio::fs;

/// 打开应用程序所在目录
#[tauri::command]
pub async fn open_app_dir() -> CmdResult<()> {
    let app_dir = dirs::app_home_dir().stringify_err()?;
    open::that(app_dir).stringify_err()
}

/// 打开核心所在目录
#[tauri::command]
pub async fn open_core_dir() -> CmdResult<()> {
    let core_dir = tauri::utils::platform::current_exe().stringify_err()?;
    let core_dir = core_dir.parent().ok_or("failed to get core dir")?;
    open::that(core_dir).stringify_err()
}

/// 打开日志目录
#[tauri::command]
pub async fn open_logs_dir() -> CmdResult<()> {
    let log_dir = dirs::app_logs_dir().stringify_err()?;
    open::that(log_dir).stringify_err()
}

/// 打开网页链接
#[tauri::command]
pub fn open_web_url(url: String) -> CmdResult<()> {
    open::that(url.as_str()).stringify_err()
}

/// Launch the default browser with the current Clash proxy (mixed port).
/// Uses a dedicated user-data-dir (like FlClash) so the browser uses only the given proxy.
#[tauri::command]
pub async fn launch_browser_with_proxy() -> CmdResult<()> {
    let port = {
        let verge = Config::verge().await.data_arc();
        match verge.verge_mixed_port {
            Some(p) => p,
            None => Config::clash().await.data_arc().get_mixed_port(),
        }
    };
    let proxy_arg = format!("--proxy-server=http://127.0.0.1:{}", port);
    let url = "https://www.google.com";

    let app_home = dirs::app_home_dir().stringify_err()?;
    #[cfg(target_os = "windows")]
    let profile_dir = app_home.join("clash-verge-edge");
    #[cfg(not(target_os = "windows"))]
    let profile_dir = app_home.join("clash-verge-chrome");
    fs::create_dir_all(&profile_dir).await.stringify_err()?;
    let user_data_arg = format!("--user-data-dir={}", profile_dir.display());

    #[cfg(target_os = "windows")]
    {
        let mut candidates: Vec<std::path::PathBuf> = vec![
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".into(),
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".into(),
        ];
        if let Ok(local) = std::env::var("LOCALAPPDATA")
            && !local.is_empty()
        {
            candidates.push(
                std::path::PathBuf::from(local)
                    .join("Microsoft")
                    .join("Edge")
                    .join("Application")
                    .join("msedge.exe"),
            );
        }
        for exe_path in candidates {
            if exe_path.exists() {
                let status = Command::new(&exe_path)
                    .args(["--new-window", &user_data_arg, &proxy_arg, url])
                    .spawn();
                if status.is_ok() {
                    return Ok(());
                }
            }
        }
        let status = Command::new("cmd")
            .args([
                "/c",
                "start",
                "",
                "msedge",
                "--new-window",
                &user_data_arg,
                &proxy_arg,
                url,
            ])
            .spawn();
        if status.is_ok() {
            return Ok(());
        }
        Err("Could not start browser (tried Edge). Install Microsoft Edge.".into())
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .args([
                "-n",
                "-a",
                "Google Chrome",
                "--args",
                "--new-window",
                &user_data_arg,
                &proxy_arg,
                url,
            ])
            .spawn();
        if status.is_ok() {
            return Ok(());
        }
        open::that(url).stringify_err()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let browsers = ["google-chrome", "google-chrome-stable", "chromium-browser", "chromium"];
        for exe in browsers {
            let status = Command::new(exe)
                .args(["--new-window", &user_data_arg, &proxy_arg, url])
                .spawn();
            if status.is_ok() {
                return Ok(());
            }
        }
        open::that(url).stringify_err()
    }
}

// TODO 后续可以为前端提供接口，当前作为托盘菜单使用
/// 打开 Verge 最新日志
#[tauri::command]
pub async fn open_app_log() -> CmdResult<()> {
    let log_path = dirs::app_latest_log().stringify_err()?;
    #[cfg(target_os = "windows")]
    let log_path = crate::utils::help::snapshot_path(&log_path).stringify_err()?;
    open::that(log_path).stringify_err()
}

// TODO 后续可以为前端提供接口，当前作为托盘菜单使用
/// 打开 Clash 最新日志
#[tauri::command]
pub async fn open_core_log() -> CmdResult<()> {
    let log_path = dirs::clash_latest_log().stringify_err()?;
    #[cfg(target_os = "windows")]
    let log_path = crate::utils::help::snapshot_path(&log_path).stringify_err()?;
    open::that(log_path).stringify_err()
}

/// 打开/关闭开发者工具
#[tauri::command]
pub fn open_devtools(app_handle: AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if !window.is_devtools_open() {
            window.open_devtools();
        } else {
            window.close_devtools();
        }
    }
}

/// 退出应用
#[tauri::command]
pub async fn exit_app() {
    feat::quit().await;
}

/// 应用内更新前调用：释放文件锁并允许安装器立即退出主进程
#[tauri::command]
pub async fn prepare_for_update() {
    let handle = handle::Handle::global();
    handle.set_is_updating();
    handle.set_is_exiting();

    utils::server::shutdown_embedded_server();
    let _ = CoreManager::global().stop_core().await;
}

/// 应用内更新失败时恢复退出拦截
#[tauri::command]
pub fn clear_prepare_for_update() {
    let handle = handle::Handle::global();
    handle.clear_is_updating();
    handle.clear_is_exiting();
}

/// Windows：安装器已启动后强制退出当前进程，由 NSIS /R 拉起新版本
#[tauri::command]
pub fn exit_for_update() {
    if handle::Handle::global().is_updating() {
        std::process::exit(0);
    }
}

/// 重启应用
#[tauri::command]
pub async fn restart_app() -> CmdResult<()> {
    feat::restart_app().await;
    Ok(())
}

/// 获取便携版标识
#[tauri::command]
pub fn get_portable_flag() -> bool {
    *dirs::PORTABLE_FLAG.get().unwrap_or(&false)
}

/// 获取应用目录
#[tauri::command]
pub fn get_app_dir() -> CmdResult<String> {
    let app_home_dir = dirs::app_home_dir().stringify_err()?.to_string_lossy().into();
    Ok(app_home_dir)
}

/// 获取当前自启动状态
#[tauri::command]
pub fn get_auto_launch_status() -> CmdResult<bool> {
    autostart::get_launch_status().stringify_err()
}

/// 下载图标缓存
#[tauri::command]
pub async fn download_icon_cache(url: String, name: String) -> CmdResult<String> {
    feat::download_icon_cache(url, name).await
}

/// 复制图标文件
#[tauri::command]
pub async fn copy_icon_file(path: String, icon_info: feat::IconInfo) -> CmdResult<String> {
    feat::copy_icon_file(path, icon_info).await
}
