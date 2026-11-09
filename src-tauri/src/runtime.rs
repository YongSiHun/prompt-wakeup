use crate::{scheduler::{SchedulerManager, TickOutcome}, storage::{err, Result}, tray};
use chrono::{DateTime, Utc};
use std::{sync::Arc, time::{Duration, Instant}};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

type ManagerState = Arc<SchedulerManager>;
struct ClockSample { wall: DateTime<Utc>, monotonic: Instant }
impl ClockSample {
    fn new() -> Self { Self { wall: Utc::now(), monotonic: Instant::now() } }
    fn sample(&mut self) -> (DateTime<Utc>, bool) {
        let now = Utc::now();
        let elapsed = self.monotonic.elapsed();
        let wall = (now - self.wall).num_milliseconds();
        let gap = elapsed > Duration::from_secs(5) || wall < 0 || (wall - elapsed.as_millis() as i64).abs() > 2000;
        self.wall = now;
        self.monotonic = Instant::now();
        (now, gap)
    }
}
pub fn start(app: tauri::AppHandle, manager: ManagerState) {
    tauri::async_runtime::spawn(run(app, manager));
}
async fn run(app: tauri::AppHandle, manager: ManagerState) {
    let mut clock = ClockSample::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    while !manager.is_stopping() {
        interval.tick().await;
        let (now, gap) = clock.sample();
        poll_once(&app, &manager, now, gap).await;
    }
}
async fn poll_once(app: &tauri::AppHandle, manager: &ManagerState, now: DateTime<Utc>, gap: bool) {
    let worker = manager.clone();
    let result = tauri::async_runtime::spawn_blocking(move || worker.tick(now, gap)).await.map_err(err).and_then(|result| result);
    match result {
        Ok(outcome) => process_outcome(app, manager, outcome),
        Err(error) => report_error(app, format!("예약 상태 처리 실패: {error}")),
    }
}
fn process_outcome(app: &tauri::AppHandle, manager: &ManagerState, outcome: TickOutcome) {
    if outcome.changed { changed(app); }
    notify(app, manager, outcome.notifications);
    for id in outcome.due { dispatch(app.clone(), manager.clone(), id); }
}
fn dispatch(app: tauri::AppHandle, manager: ManagerState, id: String) {
    tauri::async_runtime::spawn(async move {
        let worker = manager.clone();
        let result = tauri::async_runtime::spawn_blocking(move || worker.launch(&id, false)).await.map_err(err).and_then(|result| result);
        report_launch(&app, &manager, result);
    });
}
fn report_launch(app: &tauri::AppHandle, manager: &ManagerState, result: Result<()>) {
    changed(app);
    if let Err(error) = result {
        report_error(app, error.clone());
        notify(app, manager, vec![error]);
    }
}
fn notify(app: &tauri::AppHandle, manager: &ManagerState, messages: Vec<String>) {
    if !manager.notifications_enabled() { return; }
    messages.into_iter().for_each(|message| notify_one(app, message));
}
fn notify_one(app: &tauri::AppHandle, message: String) {
    let result = app.notification().builder().title("PromptWakeup").body(message).show();
    if let Err(error) = result { report_error(app, format!("시스템 알림을 표시하지 못했습니다. OS 알림 설정을 확인하세요: {error}")); }
}
pub fn changed(app: &tauri::AppHandle) { let _ = app.emit("state-changed", ()); }
pub fn report_error(app: &tauri::AppHandle, error: String) { let _ = app.emit("backend-error", error); }
pub fn window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event { api.prevent_close(); let _ = window.hide(); }
}
pub fn app_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event { prevent_implicit_exit(app, api); }
}
fn prevent_implicit_exit(app: &tauri::AppHandle, api: tauri::ExitRequestApi) {
    if !app.state::<ManagerState>().is_stopping() { api.prevent_exit(); }
}
pub fn show_existing(app: &tauri::AppHandle, _: Vec<String>, _: String) { tray::show(app); }
