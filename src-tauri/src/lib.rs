pub mod ai_cli;
pub mod commands;
pub mod execution;
pub mod models;
mod runtime;
pub mod scheduler;
pub mod security;
pub mod storage;
pub mod terminal;
pub mod tray;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(runtime::show_existing))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::save_schedule,
            commands::schedule_action,
            commands::save_settings,
            commands::detect_clis,
            commands::detect_terminals,
            commands::manual_terminal,
            commands::manual_cli,
            commands::notification_permission,
        ])
        .setup(setup)
        .on_window_event(runtime::window_event)
        .build(tauri::generate_context!())
        .expect("PromptWakeup 초기화 실패")
        .run(runtime::app_event);
}
fn setup(app: &mut tauri::App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let repository =
        storage::StorageRepository::open().map_err(|error| startup_error(app, error))?;
    let manager =
        scheduler::SchedulerManager::open(repository).map_err(|error| startup_error(app, error))?;
    app.manage(manager.clone());
    tray::TrayManager::install(app)?;
    runtime::start(app.handle().clone(), manager);
    Ok(())
}
fn startup_error(app: &tauri::App, message: String) -> Box<dyn std::error::Error> {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    app.dialog()
        .message(message.clone())
        .title("PromptWakeup 시작 실패")
        .kind(MessageDialogKind::Error)
        .blocking_show();
    Box::new(std::io::Error::other(message))
}
