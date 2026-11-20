use super::*;

#[test]
// Test: corrupt json recovers backup.
fn corrupt_json_recovers_backup() {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    repository.save("test.json", &vec![1]).unwrap();
    repository.save("test.json", &vec![2]).unwrap();
    std::fs::write(repository.root.join("test.json"), "{bad").unwrap();
    let mut warnings = Vec::new();
    let data: Vec<i32> = repository.load("test.json", &mut warnings).unwrap();
    assert_eq!(data, vec![1]);
    assert_eq!(warnings.len(), 1);
}

#[test]
// Test: future schema is not downgraded from backup.
fn future_schema_is_not_downgraded_from_backup() {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    repository.save("test.json", &vec![1]).unwrap();
    repository.save("test.json", &vec![2]).unwrap();
    std::fs::write(
        repository.root.join("test.json"),
        r#"{"schema_version":2,"data":[3]}"#,
    )
    .unwrap();
    assert!(repository
        .load::<Vec<i32>>("test.json", &mut Vec::new())
        .is_err());
}

#[test]
// Test: path traversal run id rejected.
fn path_traversal_run_id_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    assert!(repository.run_path("../../settings", "ticket").is_err());
    assert!(repository
        .run_path(&uuid::Uuid::new_v4().to_string(), "../settings")
        .is_err());
}

#[test]
// Test: legacy notification setting is ignored.
fn legacy_notification_setting_is_ignored() {
    let settings: crate::models::Settings =
        serde_json::from_str(r#"{"notifications":true}"#).unwrap();
    assert!(serde_json::to_value(settings)
        .unwrap()
        .get("notifications")
        .is_none());
}

#[test]
// Test: default cli is optional.
fn default_cli_is_optional() {
    assert_eq!(crate::models::Settings::default().default_cli, None);
}

#[test]
// Test: unset default cli round trips.
fn unset_default_cli_round_trips() {
    use crate::models::Settings;

    for json in [r#"{}"#, r#"{"default_cli":null}"#] {
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.default_cli, None);
        let saved = serde_json::to_string(&settings).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&saved).unwrap(), settings);
    }
}

#[test]
// Test: legacy cli selection is preserved.
fn legacy_cli_selection_is_preserved() {
    use crate::models::{CliKind, Settings};

    let legacy: Settings = serde_json::from_str(r#"{"default_cli":"claude"}"#).unwrap();
    assert_eq!(legacy.default_cli, Some(CliKind::Claude));
}

#[cfg(windows)]
#[test]
// Test: storage uses process identity instead of environment.
fn storage_uses_process_identity_instead_of_environment() {
    use std::os::windows::process::CommandExt;
    const CHILD: &str = "PROMPT_WAKEUP_IDENTITY_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "storage::tests::storage_uses_process_identity_instead_of_environment",
            ])
            .env(CHILD, "1")
            .env("USERDOMAIN", "nonexistent-domain")
            .env("USERNAME", "nonexistent-user")
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        return;
    }

    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("data");
    let repository = StorageRepository::at(root.clone()).unwrap();
    repository.save("test.json", &vec![1]).unwrap();
    let reopened = StorageRepository::at(root).unwrap();
    let values: Vec<i32> = reopened.load("test.json", &mut Vec::new()).unwrap();
    assert_eq!(values, vec![1]);
}

#[test]
// Test: execution prompt survives schedule changes and repository reopen.
fn execution_prompt_survives_schedule_changes_and_repository_reopen() {
    use crate::models::{ExecutionRecord, Schedule, ScheduleStatus};
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    let mut schedule = Schedule {
        id: uuid::Uuid::new_v4().to_string(),
        input: crate::test_support::input(directory.path()),
        status: ScheduleStatus::Pending,
    };
    let original = schedule.input.prompt.clone();
    let record = ExecutionRecord::new(&schedule);
    schedule.input.prompt = "changed prompt".into();
    repository.save("executions.json", &vec![record]).unwrap();
    let reopened = StorageRepository::at(repository.root).unwrap();
    let snapshot = reopened.snapshot().unwrap();
    assert!(snapshot.schedules.is_empty());
    assert_eq!(
        snapshot.executions[0].prompt.as_deref(),
        Some(original.as_str())
    );
}

#[test]
// Test: legacy execution without prompt still loads.
fn legacy_execution_without_prompt_still_loads() {
    use crate::models::{ExecutionRecord, Schedule, ScheduleStatus};
    let directory = tempfile::tempdir().unwrap();
    let schedule = Schedule {
        id: uuid::Uuid::new_v4().to_string(),
        input: crate::test_support::input(directory.path()),
        status: ScheduleStatus::Pending,
    };
    let mut legacy = serde_json::to_value(ExecutionRecord::new(&schedule)).unwrap();
    legacy.as_object_mut().unwrap().remove("prompt");
    let record: ExecutionRecord = serde_json::from_value(legacy).unwrap();
    assert_eq!(record.prompt, None);
    assert_eq!(record.schedule_id, schedule.id);
}

#[test]
// Test: invalid persisted retention is rejected before polling.
fn invalid_persisted_retention_is_rejected_before_polling() {
    use crate::models::Settings;
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    let settings = Settings {
        retention_days: u32::MAX,
        ..Settings::default()
    };
    repository.save("settings.json", &settings).unwrap();
    assert!(repository.snapshot().is_err());
}

#[test]
// Test: retired terminal fields are ignored in legacy schedules.
fn retired_terminal_fields_are_ignored_in_legacy_schedules() {
    use crate::models::ScheduleInput;
    let directory = tempfile::tempdir().unwrap();
    let input = crate::test_support::input(directory.path());
    let mut legacy = serde_json::to_value(&input).unwrap();
    legacy["terminal"] = serde_json::json!({"kind": "external"});
    legacy["keep_open"] = true.into();
    let restored: ScheduleInput = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored, input);
}
