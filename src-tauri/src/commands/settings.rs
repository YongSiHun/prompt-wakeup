use super::{blocking, ManagerState};
use crate::{
    models::Settings,
    storage::{err, Result},
};

#[tauri::command]
pub async fn save_settings(manager: ManagerState<'_>, settings: Settings) -> Result<()> {
    let manager = manager.inner().clone();
    blocking(move || manager.settings(settings)).await
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
