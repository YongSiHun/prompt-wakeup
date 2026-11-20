mod poll;
mod reconciliation;
pub mod schedule_service;
use crate::{
    execution::{launcher, ExecutionService},
    models::*,
    security::require,
    storage::{Result, StorageRepository},
};
use chrono::{DateTime, Utc};
use poll::PollPlan;
pub use poll::{recover, TickOutcome};
use schedule_service::ScheduleService;
use std::sync::{
    atomic::{AtomicBool, AtomicI64, Ordering},
    Arc, Mutex, MutexGuard,
};

type JobLaunch = dyn Fn(&str) -> Result<()> + Send + Sync;
pub struct SchedulerManager {
    pub repository: StorageRepository,
    pub state: Mutex<Snapshot>,
    pub stopping: AtomicBool,
    job_launcher: Arc<JobLaunch>,
    probe_interval: AtomicI64,
    retention_interval: AtomicI64,
}
impl SchedulerManager {
    // I/O: 저장소·파일 열기.
    pub fn open(repository: StorageRepository) -> Result<Arc<Self>> {
        let mut state = repository.snapshot()?;
        recover(&mut state, Utc::now());
        repository.save("executions.json", &state.executions)?;
        repository.save("schedules.json", &state.schedules)?;
        repository.key(true)?;
        Ok(Arc::new(Self {
            repository,
            state: Mutex::new(state),
            stopping: AtomicBool::new(false),
            job_launcher: Arc::new(launcher::launch),
            probe_interval: AtomicI64::new(i64::MIN),
            retention_interval: AtomicI64::new(i64::MIN),
        }))
    }
    // 상태: 공유 잠금.
    fn lock(&self) -> Result<MutexGuard<'_, Snapshot>> {
        self.state.lock().map_err(|_| "상태 잠금 오류".into())
    }
    // 조회: 현재 상태 복제.
    pub fn snapshot(&self) -> Result<Snapshot> {
        Ok(self.lock()?.clone())
    }
    // 조회: CLI 탐지용 설정.
    pub fn current_settings(&self) -> Result<Settings> {
        Ok(self.lock()?.settings.clone())
    }
    // 상태: 종료 요청 여부.
    pub fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::SeqCst)
    }
    // 상태: 저장 성공 후 변경 반영.
    fn edit_state(&self, edit: impl FnOnce(&mut Snapshot) -> Result<()>) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        edit(&mut next)?;
        self.repository.save_changes(&guard, &next)?;
        *guard = next;
        Ok(())
    }
    // 상태: ID별 예약 저장.
    pub fn upsert(&self, id: Option<String>, input: ScheduleInput) -> Result<()> {
        ScheduleService::validate(&input)?;
        self.edit_state(|state| ScheduleService::upsert(state, id, input))
    }
    // 상태: 예약 동작 분기.
    pub fn action(&self, id: String, action: &str) -> Result<()> {
        if action == "run" {
            return self.launch(&id, true);
        }
        self.edit_state(|state| ScheduleService::action(state, &id, action))
    }
    // IPC: 완료 이력·출력 삭제.
    pub fn delete_execution(&self, id: &str) -> Result<()> {
        self.edit_state(|state| {
            let execution = find_execution(state, id)?;
            require(
                execution.ended_at.is_some(),
                "실행이 끝난 후 삭제할 수 있습니다.",
            )?;
            state.executions.retain(|execution| execution.id != id);
            Ok(())
        })?;
        self.repository.remove_run(id);
        Ok(())
    }
    // 설정: CLI 경로 보존·저장.
    pub fn settings(&self, settings: Settings) -> Result<()> {
        crate::storage::settings_service::SettingsService::validate(&settings)?;
        self.edit_state(|state| {
            let mut settings = settings;
            settings.custom_cli_paths = state.settings.custom_cli_paths.clone();
            state.settings = settings;
            Ok(())
        })
    }
    // 설정: CLI 경로 보존.
    pub fn remember_cli(&self, kind: CliKind, path: String) -> Result<()> {
        self.edit_state(|state| {
            state
                .settings
                .custom_cli_paths
                .insert(kind.name().into(), path);
            Ok(())
        })
    }
    // 실행: 작업 프로세스 시작.
    pub fn launch(&self, id: &str, manual: bool) -> Result<()> {
        let (schedule, execution_id) = self.claim(id, manual)?;
        // OS dialogs and terminal startup must never hold the scheduler state lock.
        let result = ExecutionService::prepare(&self.repository, &schedule, &execution_id)
            .and_then(|_| (self.job_launcher)(&execution_id));
        self.finish_launch(id, &execution_id, &result)?;
        result
    }
    // 실행: 중복 시작 차단.
    fn claim(&self, id: &str, manual: bool) -> Result<(Schedule, String)> {
        let mut guard = self.lock()?;
        require(guard.active && !self.is_stopping(), "종료 중입니다.")?;
        let mut next = guard.clone();
        let schedule = ScheduleService::find(&mut next, id)?;
        ScheduleService::validate_launch(schedule, manual)?;
        let selected = schedule.clone();
        schedule.status = ScheduleStatus::Launching;
        let execution = ExecutionRecord::new(&selected);
        let execution_id = execution.id.clone();
        next.executions.push(execution);
        if let Err(error) = self.repository.save_changes(&guard, &next) {
            record_save_failure(&mut next, id, &execution_id, &error)?;
            *guard = next;
            return Err(error);
        }
        *guard = next;
        Ok((selected, execution_id))
    }
    // 상태: 래퍼 시작 결과 반영.
    fn finish_launch(&self, id: &str, execution_id: &str, result: &Result<()>) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        let execution = find_execution(&mut next, execution_id)?;
        if !execution.active() {
            return Ok(());
        }
        set_launch_result(execution, result);
        let status = execution.schedule_status();
        ScheduleService::find(&mut next, id)?.status = status;
        remove_failed_ticket(&self.repository, execution_id, result);
        let saved = self.repository.save_changes(&guard, &next);
        *guard = next;
        saved
    }
    // 상태: 예약 감시·만료 정리.
    pub fn tick(&self, now: DateTime<Utc>, gap: bool) -> Result<TickOutcome> {
        let mut guard = self.lock()?;
        let probe = interval_changed(&self.probe_interval, now.timestamp().div_euclid(5));
        let maintain = interval_changed(&self.retention_interval, now.timestamp().div_euclid(60));
        let plan = PollPlan::build(&self.repository, &guard, now, gap, probe, maintain);
        let outcome = plan.outcome();
        if plan.changed() {
            let mut next = guard.clone();
            plan.apply(&mut next);
            self.repository.save_changes(&guard, &next)?;
            *guard = next;
        }
        drop(guard);
        plan.expired
            .iter()
            .for_each(|id| self.repository.remove_run(id));
        Ok(outcome)
    }
    // 상태: 저장 후 감시 종료.
    pub fn stop(&self) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        next.active = false;
        self.save(&next)?;
        *guard = next;
        self.stopping.store(true, Ordering::SeqCst);
        Ok(())
    }
    // I/O: 백업·원자적 저장.
    fn save(&self, state: &Snapshot) -> Result<()> {
        self.repository.save("executions.json", &state.executions)?;
        self.repository.save("schedules.json", &state.schedules)?;
        self.repository.save("settings.json", &state.settings)
    }
}
// 상태: 주기 변경 감지.
fn interval_changed(interval: &AtomicI64, next: i64) -> bool {
    interval.swap(next, Ordering::Relaxed) != next
}
// 조회: ID별 실행 기록.
fn find_execution<'a>(state: &'a mut Snapshot, id: &str) -> Result<&'a mut ExecutionRecord> {
    state
        .executions
        .iter_mut()
        .find(|execution| execution.id == id)
        .ok_or_else(|| "실행 기록을 찾을 수 없습니다.".into())
}
// 상태: 실행 전 저장 실패 보존.
fn record_save_failure(
    state: &mut Snapshot,
    id: &str,
    execution_id: &str,
    error: &str,
) -> Result<()> {
    ScheduleService::find(state, id)?.status = ScheduleStatus::Failed;
    let execution = find_execution(state, execution_id)?;
    execution.status = ExecutionStatus::Failed;
    execution.ended_at = Some(Utc::now());
    execution.error = Some(format!(
        "실행 전 저장 실패로 작업을 시작하지 않았습니다: {error}"
    ));
    state.warnings.push(
        "저장 실패로 일부 상태는 메모리에만 있습니다. 디스크 공간과 파일 권한을 확인하세요.".into(),
    );
    Ok(())
}
// 상태: 실행 시작·실패 반영.
fn set_launch_result(execution: &mut ExecutionRecord, result: &Result<()>) {
    match result {
        Ok(()) => execution.status = ExecutionStatus::Launched,
        Err(error) => {
            execution.status = ExecutionStatus::Failed;
            execution.ended_at = Some(Utc::now());
            execution.error = Some(error.clone());
        }
    }
}
// 실행: 실패 요청 정리.
fn remove_failed_ticket(repository: &StorageRepository, id: &str, result: &Result<()>) {
    if result.is_err() {
        repository.remove_run_file(id, "ticket");
    }
}
#[cfg(test)]
mod tests;
