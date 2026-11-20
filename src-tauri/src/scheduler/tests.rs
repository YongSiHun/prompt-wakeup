use super::*;
use crate::test_support::{input, local_time};

// Test: setup.
fn setup() -> (tempfile::TempDir, Arc<SchedulerManager>) {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    (directory, SchedulerManager::open(repository).unwrap())
}
// Test: dispatch due.
fn dispatch_due(manager: &SchedulerManager) {
    for id in manager.tick(Utc::now(), false).unwrap().due {
        manager.launch(&id, false).unwrap();
    }
}

#[test]
// Test: create edit cancel delete preserves history.
fn create_edit_cancel_delete_preserves_history() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let first = manager.snapshot().unwrap().schedules[0].clone();
    let mut updated = first.input;
    updated.name = "수정".into();
    manager.upsert(Some(first.id.clone()), updated).unwrap();
    assert_eq!(manager.snapshot().unwrap().schedules[0].input.name, "수정");
    manager.action(first.id.clone(), "cancel").unwrap();
    assert_eq!(
        manager.snapshot().unwrap().schedules[0].status,
        ScheduleStatus::Canceled
    );
    manager.action(first.id, "delete").unwrap();
    assert!(manager.snapshot().unwrap().schedules.is_empty());
}
#[test]
// Test: simultaneous schedules remain independent.
fn simultaneous_schedules_remain_independent() {
    let (directory, manager) = setup();
    let input = input(directory.path());
    manager.upsert(None, input.clone()).unwrap();
    let mut second = input;
    second.name = "두 번째 작업".into();
    manager.upsert(None, second).unwrap();
    let state = manager.snapshot().unwrap();
    assert_eq!(state.schedules.len(), 2);
    assert_ne!(state.schedules[0].id, state.schedules[1].id);
}
#[test]
// Test: duplicate running job is rejected.
fn duplicate_running_job_is_rejected() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let id = {
        let mut state = manager.state.lock().unwrap();
        state.schedules[0].status = ScheduleStatus::Launched;
        state.schedules[0].id.clone()
    };
    assert!(manager.launch(&id, true).is_err());
    assert!(manager.snapshot().unwrap().executions.is_empty());
}
#[test]
// Test: restart restores future and marks past missed.
fn restart_restores_future_and_marks_past_missed() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let mut state = manager.snapshot().unwrap();
    state.schedules.push(state.schedules[0].clone());
    state.schedules[1].id = uuid::Uuid::new_v4().to_string();
    state.schedules[1].input.scheduled_at = Utc::now() - chrono::Duration::seconds(10);
    manager.save(&state).unwrap();
    let reopened = SchedulerManager::open(manager.repository.clone())
        .unwrap()
        .snapshot()
        .unwrap();
    assert_eq!(reopened.schedules[0].status, ScheduleStatus::Pending);
    assert_eq!(reopened.schedules[1].status, ScheduleStatus::Missed);
}
#[test]
// Test: resume gap marks due missed without launch.
fn resume_gap_marks_due_missed_without_launch() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let time = manager.snapshot().unwrap().schedules[0].input.scheduled_at;
    manager.tick(time, true).unwrap();
    assert_eq!(
        manager.snapshot().unwrap().schedules[0].status,
        ScheduleStatus::Missed
    );
    assert!(manager.snapshot().unwrap().executions.is_empty());
}
#[test]
// Test: result sets completion only after cli zero.
fn result_sets_completion_only_after_cli_zero() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let schedule = manager.snapshot().unwrap().schedules[0].clone();
    let execution = ExecutionRecord::new(&schedule);
    let id = execution.id.clone();
    {
        let mut state = manager.state.lock().unwrap();
        state.schedules[0].status = ScheduleStatus::Launched;
        state.executions.push(execution);
    }
    let result = RunResult {
        execution_id: id.clone(),
        ended_at: Utc::now(),
        exit_code: Some(0),
        error: None,
    };
    crate::storage::atomic(
        &manager.repository.run_path(&id, "result").unwrap(),
        &result,
    )
    .unwrap();
    assert!(manager.tick(Utc::now(), false).unwrap().changed);
    assert_eq!(
        manager.snapshot().unwrap().schedules[0].status,
        ScheduleStatus::Completed
    );
    assert!(!manager.tick(Utc::now(), false).unwrap().changed);
    manager.action(schedule.id, "delete").unwrap();
    assert_eq!(manager.snapshot().unwrap().executions.len(), 1);
}
#[test]
// Test: simultaneous due jobs launch once each.
fn simultaneous_due_jobs_launch_once_each() {
    let (directory, mut manager) = setup();
    let launches = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = launches.clone();
    Arc::get_mut(&mut manager).unwrap().job_launcher = Arc::new(move |_| {
        count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let input = input(directory.path());
    manager.upsert(None, input.clone()).unwrap();
    let mut second = input;
    second.name = "두 번째 작업".into();
    manager.upsert(None, second).unwrap();
    let now = DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap();
    {
        let mut state = manager.state.lock().unwrap();
        for schedule in &mut state.schedules {
            schedule.input.scheduled_at = now;
            schedule.input.local_time = local_time(now);
        }
    }
    dispatch_due(&manager);
    dispatch_due(&manager);
    assert_eq!(launches.load(Ordering::SeqCst), 2);
    assert_eq!(manager.snapshot().unwrap().executions.len(), 2);
}
#[test]
// Test: changed future time cannot launch from stale due list.
fn changed_future_time_cannot_launch_from_stale_due_list() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    assert!(manager.launch(&id, false).is_err());
    assert!(manager.snapshot().unwrap().executions.is_empty());
}
#[test]
// Test: persistence failure does not automatically retry launch.
fn persistence_failure_does_not_automatically_retry_launch() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    let path = manager.repository.root.join("executions.json");
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert!(manager.launch(&id, true).is_err());
    assert_failed_execution(&manager);
    dispatch_due(&manager);
    assert_eq!(manager.snapshot().unwrap().executions.len(), 1);
}
// Test: assert failed execution.
fn assert_failed_execution(manager: &SchedulerManager) {
    let state = manager.snapshot().unwrap();
    assert_eq!(state.schedules[0].status, ScheduleStatus::Failed);
    assert_eq!(state.executions.len(), 1);
    assert_eq!(state.executions[0].status, ExecutionStatus::Failed);
}
#[test]
// Test: idle poll does not write or request refresh.
fn idle_poll_does_not_write_or_request_refresh() {
    let (_directory, manager) = setup();
    let path = manager.repository.root.join("schedules.json");
    std::fs::remove_file(&path).unwrap();
    let outcome = manager.tick(Utc::now(), false).unwrap();
    assert!(!outcome.changed);
    assert!(outcome.due.is_empty());
    assert!(!path.exists());
}
#[test]
// Test: settings change does not rewrite schedule backup.
fn settings_change_does_not_rewrite_schedule_backup() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let path = manager.repository.root.join("schedules.json.bak");
    let previous = std::fs::read(&path).unwrap();
    let mut settings = manager.current_settings().unwrap();
    settings.theme = "dark".into();
    manager.settings(settings).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), previous);
}
#[test]
// Test: worker startup can observe state and cannot overwrite completion.
fn worker_startup_can_observe_state_and_cannot_overwrite_completion() {
    let (directory, mut manager) = setup();
    let owner = Arc::new(Mutex::new(std::sync::Weak::<SchedulerManager>::new()));
    let observer = owner.clone();
    Arc::get_mut(&mut manager).unwrap().job_launcher = Arc::new(move |id| {
        let manager = observer.lock().unwrap().upgrade().unwrap();
        assert!(
            manager.state.try_lock().is_ok(),
            "worker startup must not hold state lock"
        );
        let result = RunResult {
            execution_id: id.into(),
            ended_at: Utc::now(),
            exit_code: Some(0),
            error: None,
        };
        crate::storage::atomic(&manager.repository.run_path(id, "result").unwrap(), &result)
            .unwrap();
        manager.tick(Utc::now(), false).unwrap();
        Ok(())
    });
    *owner.lock().unwrap() = Arc::downgrade(&manager);
    manager.upsert(None, input(directory.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    manager.launch(&id, true).unwrap();
    assert_eq!(
        manager.snapshot().unwrap().schedules[0].status,
        ScheduleStatus::Completed
    );
    assert!(!manager.tick(Utc::now(), false).unwrap().changed);
}

#[test]
// Test: delete execution preserves schedule and removes saved output.
fn delete_execution_preserves_schedule_and_removes_saved_output() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let schedule = manager.snapshot().unwrap().schedules[0].clone();
    let mut execution = ExecutionRecord::new(&schedule);
    execution.status = ExecutionStatus::Completed;
    execution.ended_at = Some(Utc::now());
    let id = execution.id.clone();
    manager
        .edit_state(|state| {
            state.executions.push(execution);
            Ok(())
        })
        .unwrap();
    let output = manager.repository.run_path(&id, "output").unwrap();
    std::fs::write(&output, "saved output").unwrap();
    manager.delete_execution(&id).unwrap();
    assert!(!output.exists());
    assert!(manager.snapshot().unwrap().executions.is_empty());
    assert!(manager.repository.snapshot().unwrap().executions.is_empty());
    assert_eq!(manager.snapshot().unwrap().schedules, vec![schedule]);
}

#[test]
// Test: delete running execution is rejected.
fn delete_running_execution_is_rejected() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let schedule = manager.snapshot().unwrap().schedules[0].clone();
    let execution = ExecutionRecord::new(&schedule);
    let id = execution.id.clone();
    manager
        .edit_state(|state| {
            state.executions.push(execution);
            Ok(())
        })
        .unwrap();
    assert!(manager.delete_execution(&id).is_err());
    assert_eq!(manager.snapshot().unwrap().executions.len(), 1);
}

#[test]
// Test: duplicate names survive restart and actions use ids.
fn duplicate_names_survive_restart_and_actions_use_ids() {
    let (directory, manager) = setup();
    let mut first = input(directory.path());
    first.name = "새 예약 작업".into();
    manager.upsert(None, first.clone()).unwrap();
    first.name = " 새 예약 작업 ".into();
    manager.upsert(None, first).unwrap();
    let saved = manager.snapshot().unwrap().schedules[0].clone();
    manager.action(saved.id.clone(), "cancel").unwrap();
    let reopened = SchedulerManager::open(manager.repository.clone()).unwrap();
    let state = reopened.snapshot().unwrap();
    assert_eq!(
        state
            .schedules
            .iter()
            .map(|schedule| schedule.status)
            .collect::<Vec<_>>(),
        vec![ScheduleStatus::Canceled, ScheduleStatus::Pending],
    );
    assert_eq!(state.schedules[0].input.name, state.schedules[1].input.name);
    let second = state.schedules[1].clone();
    let mut renamed = second.input;
    renamed.name = "사용자 이름".into();
    reopened.upsert(Some(second.id), renamed).unwrap();
    assert_eq!(
        reopened.snapshot().unwrap().schedules[0].input.name,
        "새 예약 작업"
    );
}
