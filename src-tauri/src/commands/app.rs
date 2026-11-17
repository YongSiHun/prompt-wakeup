use super::ManagerState;
use crate::{
    models::Snapshot,
    scheduler::SchedulerManager,
    storage::{err, Result},
};
use std::sync::Arc;

#[tauri::command]
pub fn snapshot(manager: ManagerState<'_>) -> Result<Snapshot> {
    manager.snapshot()
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
