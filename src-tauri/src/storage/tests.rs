use super::*;

#[test]
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
fn path_traversal_run_id_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    assert!(repository.run_path("../../settings", "ticket").is_err());
    assert!(repository
        .run_path(&uuid::Uuid::new_v4().to_string(), "../settings")
        .is_err());
}

#[test]
fn legacy_notification_setting_is_ignored() {
    let settings: crate::models::Settings =
        serde_json::from_str(r#"{"notifications":true}"#).unwrap();
    assert!(serde_json::to_value(settings)
        .unwrap()
        .get("notifications")
        .is_none());
}
