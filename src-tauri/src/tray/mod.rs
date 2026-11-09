use crate::{runtime, scheduler::SchedulerManager};
use std::sync::Arc;
use tauri::{Emitter, Manager, menu::{IsMenuItem, Menu, MenuEvent, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent}};
pub fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") { let _ = window.show(); let _ = window.unminimize(); let _ = window.set_focus(); }
}
pub struct TrayManager;
impl TrayManager {
    pub fn install(app: &tauri::App) -> tauri::Result<()> {
        let menu = menu(app)?;
        let icon = app.default_window_icon().cloned().unwrap_or_else(|| tauri::image::Image::new_owned(vec![88, 85, 217, 255], 1, 1));
        TrayIconBuilder::with_id("main-tray").icon(icon).tooltip("PromptWakeup — 예약 감시 중")
            .menu(&menu).show_menu_on_left_click(false).on_menu_event(menu_event).on_tray_icon_event(tray_event).build(app)?;
        Ok(())
    }
}
fn menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let entries = [("open", "PromptWakeup 열기"), ("schedules", "예약 작업 보기 / 다음 예약 확인"), ("status", "스케줄러 상태 확인"), ("quit", "완전히 종료")];
    let items = entries.into_iter().map(|(id, label)| MenuItem::with_id(app, id, label, true, None::<&str>)).collect::<tauri::Result<Vec<_>>>()?;
    let references: Vec<&dyn IsMenuItem<tauri::Wry>> = items.iter().map(|item| item as &dyn IsMenuItem<tauri::Wry>).collect();
    Menu::with_items(app, &references)
}
fn menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    match event.id.as_ref() { "quit" => quit(app), "schedules" | "status" => navigate(app, event.id.as_ref()), _ => show(app) }
}
fn navigate(app: &tauri::AppHandle, page: &str) { show(app); let _ = app.emit("navigate", page); }
fn quit(app: &tauri::AppHandle) {
    match app.state::<Arc<SchedulerManager>>().stop() {
        Ok(()) => app.exit(0),
        Err(error) => { show(app); runtime::report_error(app, error); }
    }
}
fn tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event { show(tray.app_handle()); }
}
