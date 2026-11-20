use crate::{runtime, scheduler::SchedulerManager};
use std::sync::Arc;
use tauri::{
    menu::{IsMenuItem, Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
// UI: 창 복원·활성화.
pub fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
pub struct TrayManager;
impl TrayManager {
    // OS: 트레이 설치.
    pub fn install(app: &tauri::App) -> tauri::Result<()> {
        let menu = menu(app)?;
        let icon = app
            .default_window_icon()
            .cloned()
            .unwrap_or_else(|| tauri::image::Image::new_owned(vec![88, 85, 217, 255], 1, 1));
        TrayIconBuilder::with_id("main-tray")
            .icon(icon)
            .tooltip("PromptWakeup — 예약 감시 중")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(menu_event)
            .on_tray_icon_event(tray_event)
            .build(app)?;
        Ok(())
    }
}
// UI: 트레이 메뉴 구성.
fn menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let entries = [
        ("open", "PromptWakeup 열기"),
        ("schedules", "예약 작업 보기 / 다음 예약 확인"),
        ("executions", "실행이력 보기"),
        ("quit", "완전히 종료"),
    ];
    let items = entries
        .into_iter()
        .map(|(id, label)| MenuItem::with_id(app, id, label, true, None::<&str>))
        .collect::<tauri::Result<Vec<_>>>()?;
    let references: Vec<&dyn IsMenuItem<tauri::Wry>> = items
        .iter()
        .map(|item| item as &dyn IsMenuItem<tauri::Wry>)
        .collect();
    Menu::with_items(app, &references)
}
// UI: 트레이 메뉴 동작.
fn menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "quit" => quit(app),
        "schedules" | "executions" => navigate(app, event.id.as_ref()),
        _ => show(app),
    }
}
// UI: 화면 이동.
fn navigate(app: &tauri::AppHandle, page: &str) {
    show(app);
    let _ = app.emit("navigate", page);
}
// 실행: 저장 후 앱 종료.
fn quit(app: &tauri::AppHandle) {
    match app.state::<Arc<SchedulerManager>>().stop() {
        Ok(()) => app.exit(0),
        Err(error) => {
            show(app);
            runtime::report_error(app, error);
        }
    }
}
// UI: 트레이 클릭 처리.
fn tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show(tray.app_handle());
    }
}
