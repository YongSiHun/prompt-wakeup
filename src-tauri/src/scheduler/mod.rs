pub mod schedule_service;
mod poll;
mod reconciliation;
pub use poll::{recover, TickOutcome};
use crate::{execution::ExecutionService, models::*, security::require, storage::{Result, StorageRepository}, terminal::TerminalLauncher};
use chrono::{DateTime, Utc};
use poll::PollPlan;
use schedule_service::ScheduleService;
use std::sync::{Arc, Mutex, MutexGuard, atomic::{AtomicBool, AtomicI64, Ordering}};

type TerminalLaunch = dyn Fn(&TerminalInfo, &str) -> Result<()> + Send + Sync;
pub struct SchedulerManager {
    pub repository: StorageRepository,
    pub state: Mutex<Snapshot>,
    pub stopping: AtomicBool,
    terminal_launcher: Arc<TerminalLaunch>,
    probe_interval: AtomicI64,
    retention_interval: AtomicI64,
}
impl SchedulerManager {
    pub fn open(repository: StorageRepository) -> Result<Arc<Self>> {
        let mut state = repository.snapshot()?;
        recover(&mut state, Utc::now());
        repository.save("executions.json", &state.executions)?;
        repository.save("schedules.json", &state.schedules)?;
        repository.key(true)?;
        Ok(Arc::new(Self {
            repository, state: Mutex::new(state), stopping: AtomicBool::new(false),
            terminal_launcher: Arc::new(TerminalLauncher::launch),
            probe_interval: AtomicI64::new(i64::MIN), retention_interval: AtomicI64::new(i64::MIN),
        }))
    }
    fn lock(&self) -> Result<MutexGuard<'_, Snapshot>> { self.state.lock().map_err(|_| "상태 잠금 오류".into()) }
    pub fn snapshot(&self) -> Result<Snapshot> { Ok(self.lock()?.clone()) }
    pub fn current_settings(&self) -> Result<Settings> { Ok(self.lock()?.settings.clone()) }
    pub fn notifications_enabled(&self) -> bool { self.lock().is_ok_and(|state| state.settings.notifications) }
    pub fn is_stopping(&self) -> bool { self.stopping.load(Ordering::SeqCst) }
    fn edit_state(&self, edit: impl FnOnce(&mut Snapshot) -> Result<()>) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        edit(&mut next)?;
        self.repository.save_changes(&guard, &next)?;
        *guard = next;
        Ok(())
    }
    pub fn upsert(&self, id: Option<String>, input: ScheduleInput) -> Result<()> {
        ScheduleService::validate(&input)?;
        self.edit_state(|state| ScheduleService::upsert(state, id, input))
    }
    pub fn action(&self, id: String, action: &str) -> Result<()> {
        if action == "run" { return self.launch(&id, true); }
        self.edit_state(|state| ScheduleService::action(state, &id, action))
    }
    pub fn settings(&self, settings: Settings) -> Result<()> {
        crate::storage::settings_service::SettingsService::validate(&settings)?;
        self.edit_state(|state| {
            let mut settings=settings;
            settings.custom_cli_paths=state.settings.custom_cli_paths.clone();
            settings.custom_terminals=state.settings.custom_terminals.clone();
            state.settings=settings;
            Ok(())
        })
    }
    pub fn remember_terminal(&self, terminal: TerminalInfo) -> Result<()> {
        self.edit_state(|state| {
            state.settings.custom_terminals.retain(|item| item.id != terminal.id);
            state.settings.custom_terminals.push(terminal);
            crate::storage::settings_service::SettingsService::validate(&state.settings)
        })
    }
    pub fn remember_cli(&self, kind: CliKind, path: String) -> Result<()> {
        self.edit_state(|state| { state.settings.custom_cli_paths.insert(kind.name().into(), path); Ok(()) })
    }
    pub fn launch(&self, id: &str, manual: bool) -> Result<()> {
        let (schedule, execution_id) = self.claim(id, manual)?;
        // OS dialogs and terminal startup must never hold the scheduler state lock.
        let result = ExecutionService::prepare(&self.repository, &schedule, &execution_id)
            .and_then(|_| (self.terminal_launcher)(&schedule.input.terminal, &execution_id));
        self.finish_launch(id, &execution_id, &result)?;
        result
    }
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
    fn finish_launch(&self, id: &str, execution_id: &str, result: &Result<()>) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        let execution = find_execution(&mut next, execution_id)?;
        if !execution.active() { return Ok(()); }
        set_launch_result(execution, result);
        let status = execution.schedule_status();
        ScheduleService::find(&mut next, id)?.status = status;
        remove_failed_ticket(&self.repository, execution_id, result);
        let saved = self.repository.save_changes(&guard, &next);
        *guard = next;
        saved
    }
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
        plan.expired.iter().for_each(|id| self.repository.remove_run(id));
        Ok(outcome)
    }
    pub fn stop(&self) -> Result<()> {
        let mut guard = self.lock()?;
        let mut next = guard.clone();
        next.active = false;
        self.save(&next)?;
        *guard = next;
        self.stopping.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn save(&self, state: &Snapshot) -> Result<()> {
        self.repository.save("executions.json", &state.executions)?;
        self.repository.save("schedules.json", &state.schedules)?;
        self.repository.save("settings.json", &state.settings)
    }
}
fn interval_changed(interval: &AtomicI64, next: i64) -> bool { interval.swap(next, Ordering::Relaxed) != next }
fn find_execution<'a>(state: &'a mut Snapshot, id: &str) -> Result<&'a mut ExecutionRecord> {
    state.executions.iter_mut().find(|execution| execution.id == id).ok_or_else(|| "실행 기록을 찾을 수 없습니다.".into())
}
fn record_save_failure(state: &mut Snapshot, id: &str, execution_id: &str, error: &str) -> Result<()> {
    ScheduleService::find(state, id)?.status = ScheduleStatus::Failed;
    let execution = find_execution(state, execution_id)?;
    execution.status = ExecutionStatus::Failed;
    execution.ended_at = Some(Utc::now());
    execution.error = Some(format!("실행 전 저장 실패로 작업을 시작하지 않았습니다: {error}"));
    state.warnings.push("저장 실패로 일부 상태는 메모리에만 있습니다. 디스크 공간과 파일 권한을 확인하세요.".into());
    Ok(())
}
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
fn remove_failed_ticket(repository: &StorageRepository, id: &str, result: &Result<()>) {
    if result.is_err() { repository.remove_run_file(id, "ticket"); }
}
#[cfg(test)]
pub(crate) mod tests;
