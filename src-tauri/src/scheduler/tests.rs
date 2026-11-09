use super::*;
use crate::{ai_cli::AiCliAdapter, security::SecurityValidator};

fn setup() -> (tempfile::TempDir, Arc<SchedulerManager>) {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    (directory, SchedulerManager::open(repository).unwrap())
}
pub(crate) fn input(directory: &std::path::Path) -> ScheduleInput {
    let cli = directory.join(if cfg!(windows) { "codex.cmd" } else { "codex" });
    std::fs::write(&cli, version_fixture()).unwrap();
    make_executable(&cli);
    let terminal = crate::terminal::detection::TerminalDetector::detect()
        .into_iter()
        .find(|terminal| terminal.supported)
        .unwrap();
    let time =
        DateTime::from_timestamp((Utc::now() + chrono::Duration::hours(1)).timestamp(), 0).unwrap();
    ScheduleInput {
        name: "한글 예약".into(),
        cli: CliKind::Codex,
        cli_path: cli.to_string_lossy().into(),
        terminal,
        working_directory: directory.to_string_lossy().into(),
        prompt: "한글 ' \" & $(touch bad) \n다음 줄 %PATH%".into(),
        scheduled_at: time,
        timezone: "Asia/Seoul".into(),
        local_time: local_time(time),
        keep_open: false,
    }
}
fn local_time(time: DateTime<Utc>) -> String {
    time.with_timezone(&chrono_tz::Asia::Seoul)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}
fn version_fixture() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\necho fake-cli 1.0\r\nexit /b 0\r\n"
    } else {
        "#!/bin/sh\necho fake-cli 1.0\n"
    }
}
#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
#[cfg(not(unix))]
fn make_executable(_: &std::path::Path) {}
fn dispatch_due(manager: &SchedulerManager) {
    for id in manager.tick(Utc::now(), false).unwrap().due {
        manager.launch(&id, false).unwrap();
    }
}

#[test]
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
fn simultaneous_schedules_remain_independent() {
    let (directory, manager) = setup();
    let input = input(directory.path());
    manager.upsert(None, input.clone()).unwrap();
    manager.upsert(None, input).unwrap();
    let state = manager.snapshot().unwrap();
    assert_eq!(state.schedules.len(), 2);
    assert_ne!(state.schedules[0].id, state.schedules[1].id);
}
#[test]
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
fn utc_local_roundtrip_and_dst_rejection() {
    let (directory, _manager) = setup();
    let mut input = input(directory.path());
    assert!(SecurityValidator::input(&input, true).is_ok());
    input.timezone = "America/New_York".into();
    input.local_time = "2027-11-07T01:30:00".into();
    input.scheduled_at = "2027-11-07T05:30:00Z".parse().unwrap();
    assert!(SecurityValidator::input(&input, false)
        .unwrap_err()
        .contains("DST"));
    input.local_time = "2027-03-14T02:30:00".into();
    assert!(SecurityValidator::input(&input, false).is_err());
}
#[test]
fn invalid_path_and_empty_prompt_rejected() {
    let (directory, _manager) = setup();
    let mut input = input(directory.path());
    input.prompt = " ".into();
    assert!(SecurityValidator::input(&input, true).is_err());
    input.prompt = "ok".into();
    input.cli_path = "relative/codex".into();
    assert!(SecurityValidator::input(&input, true).is_err());
}
#[test]
fn adapters_keep_prompts_out_of_arguments() {
    let (directory, _manager) = setup();
    let input = input(directory.path());
    let plans = [
        crate::ai_cli::adapters::CodexAdapter.build_launch_plan(&input),
        crate::ai_cli::adapters::ClaudeAdapter.build_launch_plan(&input),
        crate::ai_cli::adapters::GeminiAdapter.build_launch_plan(&input),
    ];
    assert_adapter_arguments(&plans);
    for plan in plans {
        assert_prompt_delivery(&plan, &input.prompt);
    }
}
fn assert_adapter_arguments(plans: &[LaunchPlan; 3]) {
    assert_eq!(plans[0].arguments, vec!["exec", "-"]);
    assert_eq!(plans[1].arguments, vec!["-p"]);
    assert_eq!(
        plans[2].arguments,
        vec!["-p", "Execute the task supplied on stdin"]
    );
}
fn assert_prompt_delivery(plan: &LaunchPlan, prompt: &str) {
    assert!(!plan.arguments.iter().any(|argument| argument == prompt));
    assert!(plan.environment.is_empty());
    assert_eq!(plan.prompt_delivery, "stdin_utf8");
}
#[test]
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
fn corrupt_json_recovers_backup() {
    let (_directory, manager) = setup();
    manager.repository.save("test.json", &vec![1]).unwrap();
    manager.repository.save("test.json", &vec![2]).unwrap();
    std::fs::write(manager.repository.root.join("test.json"), "{bad").unwrap();
    let mut warnings = Vec::new();
    let data: Vec<i32> = manager.repository.load("test.json", &mut warnings).unwrap();
    assert_eq!(data, vec![1]);
    assert_eq!(warnings.len(), 1);
}
#[test]
fn future_schema_is_not_downgraded_from_backup() {
    let (_directory, manager) = setup();
    manager.repository.save("test.json", &vec![1]).unwrap();
    manager.repository.save("test.json", &vec![2]).unwrap();
    std::fs::write(
        manager.repository.root.join("test.json"),
        r#"{"schema_version":2,"data":[3]}"#,
    )
    .unwrap();
    assert!(manager
        .repository
        .load::<Vec<i32>>("test.json", &mut Vec::new())
        .is_err());
}
#[test]
fn path_traversal_run_id_rejected() {
    let (_directory, manager) = setup();
    assert!(manager
        .repository
        .run_path("../../settings", "ticket")
        .is_err());
    assert!(manager
        .repository
        .run_path(&uuid::Uuid::new_v4().to_string(), "../settings")
        .is_err());
}
#[test]
fn shell_quotes_do_not_expose_prompt_as_code() {
    assert_eq!(crate::security::ps_quote("a'b"), "'a''b'");
    assert_eq!(crate::security::sh_quote("a'b"), "'a'\"'\"'b'");
}
#[test]
fn version_detects_stub_cli() {
    let (directory, _manager) = setup();
    assert!(crate::ai_cli::version(&input(directory.path()).cli_path)
        .unwrap()
        .contains("fake-cli"));
}
#[test]
fn simultaneous_due_jobs_launch_once_each() {
    let (directory, mut manager) = setup();
    let launches = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = launches.clone();
    Arc::get_mut(&mut manager).unwrap().terminal_launcher = Arc::new(move |_, _| {
        count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let input = input(directory.path());
    manager.upsert(None, input.clone()).unwrap();
    manager.upsert(None, input).unwrap();
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
fn changed_future_time_cannot_launch_from_stale_due_list() {
    let (directory, manager) = setup();
    manager.upsert(None, input(directory.path())).unwrap();
    let id = manager.snapshot().unwrap().schedules[0].id.clone();
    assert!(manager.launch(&id, false).is_err());
    assert!(manager.snapshot().unwrap().executions.is_empty());
}
#[test]
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
fn assert_failed_execution(manager: &SchedulerManager) {
    let state = manager.snapshot().unwrap();
    assert_eq!(state.schedules[0].status, ScheduleStatus::Failed);
    assert_eq!(state.executions.len(), 1);
    assert_eq!(state.executions[0].status, ExecutionStatus::Failed);
}
#[test]
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
fn terminal_startup_can_observe_state_and_cannot_overwrite_completion() {
    let (directory, mut manager) = setup();
    let owner = Arc::new(Mutex::new(std::sync::Weak::<SchedulerManager>::new()));
    let observer = owner.clone();
    Arc::get_mut(&mut manager).unwrap().terminal_launcher = Arc::new(move |_, id| {
        let manager = observer.lock().unwrap().upgrade().unwrap();
        assert!(
            manager.state.try_lock().is_ok(),
            "terminal startup must not hold state lock"
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
fn verbose_version_output_does_not_fill_the_pipe() {
    let (directory, _manager) = setup();
    let input = input(directory.path());
    let script = if cfg!(windows) {
        "@echo off\r\nfor /L %%i in (1,1,20000) do @echo fake-cli version output\r\nexit /b 0\r\n"
    } else {
        "#!/bin/sh\ni=0\nwhile [ $i -lt 20000 ]; do echo fake-cli version output; i=$((i+1)); done\n"
    };
    std::fs::write(&input.cli_path, script).unwrap();
    let version = crate::ai_cli::version(&input.cli_path).unwrap();
    assert!(version.starts_with("fake-cli"));
    assert!(version.len() <= 4096);
}
