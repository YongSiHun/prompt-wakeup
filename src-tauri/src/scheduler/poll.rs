use super::{reconciliation, schedule_service::ScheduleService};
use crate::{models::*, storage::StorageRepository};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct TickOutcome {
    pub due: Vec<String>,
    pub changed: bool,
}
#[derive(Default)]
pub struct PollPlan {
    pub due: Vec<String>,
    statuses: HashMap<String, ScheduleStatus>,
    executions: Vec<ExecutionRecord>,
    pub expired: HashSet<String>,
}
impl PollPlan {
    // 상태: 예약 갱신 계획.
    pub fn build(
        repository: &StorageRepository,
        state: &Snapshot,
        now: DateTime<Utc>,
        gap: bool,
        probe: bool,
        maintain: bool,
    ) -> Self {
        let mut plan = Self::default();
        plan.collect_due(state, now, gap);
        plan.collect_results(repository, state, now, probe);
        if maintain {
            plan.collect_expired(state, now);
        }
        plan
    }
    // 조회: 실행 시각 도달 예약.
    fn collect_due(&mut self, state: &Snapshot, now: DateTime<Utc>, gap: bool) {
        for schedule in state
            .schedules
            .iter()
            .filter(|schedule| ScheduleService::is_due(schedule, now))
        {
            self.classify_due(schedule, now, gap);
        }
    }
    // 상태: 실행·놓침 분류.
    fn classify_due(&mut self, schedule: &Schedule, now: DateTime<Utc>, gap: bool) {
        if gap || now - schedule.input.scheduled_at > Duration::seconds(5) {
            self.statuses
                .insert(schedule.id.clone(), ScheduleStatus::Missed);
        } else {
            self.due.push(schedule.id.clone());
        }
    }
    // I/O: 활성 실행 결과 수집.
    fn collect_results(
        &mut self,
        repository: &StorageRepository,
        state: &Snapshot,
        now: DateTime<Utc>,
        probe: bool,
    ) {
        self.executions = state
            .executions
            .iter()
            .filter(|execution| execution.active())
            .filter_map(|execution| reconciliation::observe(repository, execution, now, probe))
            .collect();
        self.statuses.extend(
            self.executions
                .iter()
                .map(|execution| (execution.schedule_id.clone(), execution.schedule_status())),
        );
    }
    // 조회: 보존 기간 만료 이력.
    fn collect_expired(&mut self, state: &Snapshot, now: DateTime<Utc>) {
        let cutoff = now - Duration::days(state.settings.retention_days.into());
        self.expired = state
            .executions
            .iter()
            .filter(|execution| execution.ended_at.is_some_and(|time| time < cutoff))
            .map(|execution| execution.id.clone())
            .collect();
    }
    // 상태: 변경 여부.
    pub fn changed(&self) -> bool {
        !self.statuses.is_empty() || !self.executions.is_empty() || !self.expired.is_empty()
    }
    // 변환: 감시 결과.
    pub fn outcome(&self) -> TickOutcome {
        TickOutcome {
            due: self.due.clone(),
            changed: self.changed(),
        }
    }
    // 상태: 변경 계획 반영.
    pub fn apply(&self, state: &mut Snapshot) {
        for schedule in &mut state.schedules {
            self.update_schedule(schedule);
        }
        let updates: HashMap<_, _> = self
            .executions
            .iter()
            .map(|execution| (execution.id.as_str(), execution))
            .collect();
        state
            .executions
            .retain(|execution| !self.expired.contains(&execution.id));
        for execution in &mut state.executions {
            replace_execution(execution, &updates);
        }
    }
    // 상태: 예약 결과 반영.
    fn update_schedule(&self, schedule: &mut Schedule) {
        if let Some(status) = self.statuses.get(&schedule.id) {
            schedule.status = *status;
        }
    }
}
// 상태: ID별 실행 결과 반영.
fn replace_execution(execution: &mut ExecutionRecord, updates: &HashMap<&str, &ExecutionRecord>) {
    if let Some(update) = updates.get(execution.id.as_str()) {
        *execution = (**update).clone();
    }
}
// 상태: 재시작 복구·중복 실행 방지.
pub fn recover(state: &mut Snapshot, now: DateTime<Utc>) {
    let running: HashSet<_> = state
        .executions
        .iter()
        .filter(|execution| execution.active())
        .map(|execution| execution.schedule_id.as_str())
        .collect();
    for schedule in &mut state.schedules {
        restore_schedule(schedule, &running, now);
    }
    drop(running);
    state
        .executions
        .iter_mut()
        .filter(|execution| execution.active())
        .for_each(restore_execution);
}
// 상태: 예약 추적·놓침 복구.
fn restore_schedule(schedule: &mut Schedule, running: &HashSet<&str>, now: DateTime<Utc>) {
    if running.contains(schedule.id.as_str()) || schedule.status.is_running() {
        schedule.status = ScheduleStatus::Launched;
    } else if ScheduleService::is_due(schedule, now) {
        schedule.status = ScheduleStatus::Missed;
    }
}
// 상태: 실행 결과 재확인.
fn restore_execution(execution: &mut ExecutionRecord) {
    execution.status = ExecutionStatus::Unknown;
    execution.error =
        Some("앱 재시작으로 진행 상태 확인 중입니다. 자동 재실행하지 않습니다.".into());
}
