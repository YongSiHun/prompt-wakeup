use crate::{
    scheduler::{SchedulerManager, TickOutcome},
    storage::{err, Result},
    tray,
};
use chrono::{DateTime, Utc};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};

type ManagerState = Arc<SchedulerManager>;
struct ClockSample {
    wall: DateTime<Utc>,
    monotonic: Instant,
}
impl ClockSample {
    // 상태: 초기값 구성.
    fn new() -> Self {
        Self {
            wall: Utc::now(),
            monotonic: Instant::now(),
        }
    }
    // 시계: 절전·시각 변경 감지.
    fn sample(&mut self) -> (DateTime<Utc>, bool) {
        let now = Utc::now();
        let elapsed = self.monotonic.elapsed();
        let wall = (now - self.wall).num_milliseconds();
        let gap = elapsed > Duration::from_secs(5)
            || wall < 0
            || (wall - elapsed.as_millis() as i64).abs() > 2000;
        self.wall = now;
        self.monotonic = Instant::now();
        (now, gap)
    }
}
// 실행: 예약 감시 시작.
pub fn start(app: tauri::AppHandle, manager: ManagerState) {
    tauri::async_runtime::spawn(run(app, manager));
}
// 실행: 앱·예약 감시.
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
// 실행: 비동기 감시 회차.
async fn poll_once(app: &tauri::AppHandle, manager: &ManagerState, now: DateTime<Utc>, gap: bool) {
    let worker = manager.clone();
    let result = blocking(move || worker.tick(now, gap)).await;
    match result {
        Ok(outcome) => process_outcome(app, manager, outcome),
        Err(error) => report_error(app, format!("예약 상태 처리 실패: {error}")),
    }
}
// 상태: 변경 통지·예약 실행.
fn process_outcome(app: &tauri::AppHandle, manager: &ManagerState, outcome: TickOutcome) {
    if outcome.changed {
        changed(app);
    }
    for id in outcome.due {
        dispatch(app.clone(), manager.clone(), id);
    }
}
// 실행: 예약 작업 분리.
fn dispatch(app: tauri::AppHandle, manager: ManagerState, id: String) {
    tauri::async_runtime::spawn(async move {
        let result = blocking(move || manager.launch(&id, false)).await;
        report_launch(&app, result);
    });
}
// IPC: 시작 결과 통지.
fn report_launch(app: &tauri::AppHandle, result: Result<()>) {
    changed(app);
    if let Err(error) = result {
        report_error(app, error);
    }
}
// 상태: 변경 여부.
pub fn changed(app: &tauri::AppHandle) {
    let _ = app.emit("state-changed", ());
}
// IPC: 오류 통지.
pub fn report_error(app: &tauri::AppHandle, error: String) {
    let _ = app.emit("backend-error", error);
}
// UI: 창 닫기 요청.
pub fn window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.emit("close-requested", ());
    }
}
// OS: 앱 종료 요청.
pub fn app_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        prevent_implicit_exit(app, api);
    }
}
// OS: 트레이 감시 유지.
fn prevent_implicit_exit(app: &tauri::AppHandle, api: tauri::ExitRequestApi) {
    if !app.state::<ManagerState>().is_stopping() {
        api.prevent_exit();
    }
}
// UI: 기존 앱 활성화.
pub fn show_existing(app: &tauri::AppHandle, _: Vec<String>, _: String) {
    tray::show(app);
}

// IPC: 블로킹 작업·오류 변환 공통 처리.
pub(crate) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(err)?
}
