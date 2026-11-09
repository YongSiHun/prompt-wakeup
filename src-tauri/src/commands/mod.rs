use crate::{
    models::*,
    scheduler::SchedulerManager,
    storage::{err, Result},
};
use std::sync::Arc;
use tauri::State;
type ManagerState<'a> = State<'a, Arc<SchedulerManager>>;
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
