use crate::{
    models::*,
    scheduler::SchedulerManager,
    storage::{err, Result},
};
use std::sync::Arc;
use tauri::State;
type ManagerState<'a> = State<'a, Arc<SchedulerManager>>;
#[tauri::command]
pub async fn delete_execution(manager: ManagerState<'_>, id: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.delete_execution(&id)).await
}
#[tauri::command]
pub fn close_app(app: tauri::AppHandle, action: String) -> Result<()> {
    use tauri::Manager;
    match action.as_str() {
        "tray" => app
            .get_webview_window("main")
            .ok_or("앱 창을 찾을 수 없습니다.")?
            .hide()
            .map_err(err),
        "quit" => {
            app.state::<Arc<SchedulerManager>>().stop()?;
            app.exit(0);
            Ok(())
        }
        _ => Err("종료 방식을 선택하세요.".into()),
    }
}
async fn blocking<T, F>(work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(err)?
}
#[tauri::command]
pub fn snapshot(manager: ManagerState<'_>) -> Result<Snapshot> {
    manager.snapshot()
}
#[tauri::command]
pub async fn save_schedule(
    manager: ManagerState<'_>,
    id: Option<String>,
    input: ScheduleInput,
) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.upsert(id, input)).await
}
#[tauri::command]
pub async fn schedule_action(manager: ManagerState<'_>, id: String, action: String) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.action(id, &action)).await
}
#[tauri::command]
pub async fn save_settings(manager: ManagerState<'_>, settings: Settings) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.settings(settings)).await
}
#[tauri::command]
pub async fn detect_clis(manager: ManagerState<'_>) -> Result<Vec<CliInfo>> {
    let settings = manager.current_settings()?;
    blocking(move || Ok(crate::ai_cli::detect(&settings))).await
}
#[tauri::command]
pub fn detect_terminals(manager: ManagerState<'_>) -> Result<Vec<TerminalInfo>> {
    Ok(crate::terminal::detection::TerminalDetector::configured(
        &manager.current_settings()?,
    ))
}
#[tauri::command]
pub async fn manual_terminal(manager: ManagerState<'_>, path: String) -> Result<TerminalInfo> {
    let manager = manager.inner().clone();
    blocking(move || {
        let terminal = crate::terminal::manual(path)?;
        crate::terminal::validate(&terminal)?;
        manager.remember_terminal(terminal.clone())?;
        Ok(terminal)
    })
    .await
}
#[tauri::command]
pub async fn manual_cli(manager: ManagerState<'_>, kind: CliKind, path: String) -> Result<CliInfo> {
    let manager = manager.inner().clone();
    blocking(move || {
        let info = crate::ai_cli::inspect(kind, path.clone());
        crate::security::require(
            info.available,
            info.validation_error
                .as_deref()
                .unwrap_or("CLI를 확인할 수 없습니다."),
        )?;
        manager.remember_cli(kind, path)?;
        Ok(info)
    })
    .await
}
#[tauri::command]
pub fn notification_permission(app: tauri::AppHandle, request: bool) -> Result<String> {
    use tauri_plugin_notification::NotificationExt;
    let result = if request {
        app.notification().request_permission()
    } else {
        app.notification().permission_state()
    };
    result
        .map(|permission| format!("{permission:?}"))
        .map_err(err)
}

#[tauri::command]
pub async fn execution_output(manager: ManagerState<'_>, id: String) -> Result<String> {
    let repository = manager.repository.clone();
    blocking(move || {
        let path = repository.run_path(&id, "output")?;
        match std::fs::read(path) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(error) => Err(err(error)),
        }
    })
    .await
}

#[tauri::command]
pub fn execution_output_path(manager: ManagerState<'_>, id: String) -> Result<String> {
    Ok(manager
        .repository
        .run_path(&id, "output")?
        .to_string_lossy()
        .into_owned())
}
#[tauri::command]
pub async fn open_execution_output(
    manager: ManagerState<'_>,
    id: String,
    folder: bool,
) -> Result<()> {
    let path = manager.repository.run_path(&id, "output")?;
    blocking(move || crate::storage::output_files::open(&path, folder)).await
}
