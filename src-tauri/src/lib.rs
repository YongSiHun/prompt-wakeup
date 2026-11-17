pub mod ai_cli;
pub mod commands;
pub mod execution;
pub mod models;
mod runtime;
pub mod scheduler;
pub mod security;
pub mod storage;
pub mod tray;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(runtime::show_existing))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::app::snapshot,
            commands::executions::execution_output,
            commands::executions::execution_output_path,
            commands::executions::open_execution_output,
            commands::executions::delete_execution,
            commands::app::close_app,
            commands::schedules::save_schedule,
            commands::schedules::schedule_action,
            commands::settings::save_settings,
            commands::clis::detect_clis,
            commands::clis::manual_cli,
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

#[cfg(test)]
mod test_support;
