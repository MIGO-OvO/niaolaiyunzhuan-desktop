use crate::{
    backup::MAX_IMAGE_BYTES,
    tracker::{RestorePreview, Snapshot},
    Tracker,
};
use chrono::Utc;
use serde::Serialize;
use std::{fs::File, io::Read, sync::Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

struct DesktopState(Mutex<Result<Tracker, String>>);

fn with_tracker<T>(
    app: &AppHandle,
    operation: impl FnOnce(&mut Tracker) -> Result<T, String>,
) -> Result<T, String> {
    let state = app.state::<DesktopState>();
    let mut guard = state.0.lock().map_err(|_| "存档状态异常，请重新打开程序")?;
    let tracker = guard.as_mut().map_err(|error| error.clone())?;
    operation(tracker)
}

#[tauri::command]
fn get_snapshot(app: AppHandle) -> Result<Snapshot, String> {
    with_tracker(&app, |tracker| {
        let now = Utc::now();
        tracker.refresh(now)?;
        tracker.snapshot(now)
    })
}

#[tauri::command]
fn set_checked(app: AppHandle, atom_ids: Vec<String>, checked: bool) -> Result<Snapshot, String> {
    with_tracker(&app, |tracker| {
        let now = Utc::now();
        let ids: Vec<_> = atom_ids.iter().map(String::as_str).collect();
        tracker.set_checked_many(&ids, checked, now)?;
        tracker.snapshot(now)
    })
}

#[tauri::command]
fn mark_seen(app: AppHandle, module: String) -> Result<Snapshot, String> {
    with_tracker(&app, |tracker| {
        let now = Utc::now();
        tracker.mark_seen(&module, now)?;
        tracker.snapshot(now)
    })
}

#[tauri::command]
fn reset_progress(app: AppHandle, module: Option<String>) -> Result<Snapshot, String> {
    with_tracker(&app, |tracker| {
        let now = Utc::now();
        tracker.reset(module.as_deref(), now)?;
        tracker.snapshot(now)
    })
}

#[tauri::command]
fn restore_backup(app: AppHandle, payload: String) -> Result<Snapshot, String> {
    with_tracker(&app, |tracker| {
        let now = Utc::now();
        tracker.restore_payload(&payload, now)?;
        tracker.snapshot(now)
    })
}

#[derive(Serialize)]
struct BackupExport {
    snapshot: Snapshot,
    path: String,
}

#[tauri::command]
async fn export_backup(app: AppHandle) -> Result<Option<BackupExport>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let name = format!(
            "袅来运转-{}.png",
            crate::state::china_time(Utc::now()).format("%Y%m%d-%H%M%S")
        );
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("PNG 备份图片", &["png"])
            .set_file_name(name)
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "请选择本机上的保存位置")?;
        with_tracker(&app, |tracker| {
            let now = Utc::now();
            tracker.export_backup(&path, now)?;
            Ok(Some(BackupExport {
                snapshot: tracker.snapshot(now)?,
                path: path.display().to_string(),
            }))
        })
    })
    .await
    .map_err(|error| format!("备份任务未完成：{error}"))?
}

#[tauri::command]
async fn preview_restore(app: AppHandle) -> Result<Option<RestorePreview>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("备份图片", &["png", "jpg", "jpeg"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "请选择本机上的备份图片")?;
        let mut bytes = Vec::new();
        File::open(path)
            .and_then(|file| {
                file.take(MAX_IMAGE_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
            })
            .map_err(|error| format!("无法读取备份图片：{error}"))?;
        with_tracker(&app, |tracker| {
            tracker.preview_restore(&bytes, Utc::now()).map(Some)
        })
    })
    .await
    .map_err(|error| format!("备份识别未完成：{error}"))?
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let tracker = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("无法定位存档目录：{error}"))
                .and_then(|path| Tracker::open(&path, Utc::now()));
            app.manage(DesktopState(Mutex::new(tracker)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            set_checked,
            mark_seen,
            reset_progress,
            restore_backup,
            export_backup,
            preview_restore
        ])
        .run(tauri::generate_context!())
        .expect("无法启动袅来运转桌面窗口");
}
