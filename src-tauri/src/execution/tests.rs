use super::*;
fn fixture(code: i32) -> (tempfile::TempDir, StorageRepository, ScheduleInput) {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    let input = crate::scheduler::tests::input(directory.path());
    let script = format!("const fs=require('node:fs');if(process.argv.includes('--version')){{console.log('fixture 1');process.exit(0)}}const data=[];process.stdin.on('data',c=>data.push(c));process.stdin.on('end',()=>{{fs.writeFileSync('received.txt',Buffer.concat(data));process.exit({code})}});");
    std::fs::write(directory.path().join("fixture.cjs"), script).unwrap();
    std::fs::write(&input.cli_path, cli_shim()).unwrap();
    (directory, repository, input)
}
fn cli_shim() -> &'static str {
    if cfg!(windows) { "@echo off\r\nnode \"%~dp0fixture.cjs\" %*\r\n" }
    else { "#!/bin/sh\nexec node \"$(dirname \"$0\")/fixture.cjs\" \"$@\"\n" }
}
fn prepare(repository: &StorageRepository, input: ScheduleInput) -> String {
    let schedule = Schedule { id: uuid::Uuid::new_v4().to_string(), input, status: ScheduleStatus::Pending };
    let id = uuid::Uuid::new_v4().to_string();
    ExecutionService::prepare(repository, &schedule, &id).unwrap();
    id
}
#[test]
fn noninteractive_stdin_preserves_unicode_quotes_newlines() {
    let (directory, _repository, input) = fixture(0);
    assert_eq!(execute(&input).unwrap(), Some(0));
    assert_eq!(std::fs::read_to_string(directory.path().join("received.txt")).unwrap(), input.prompt);
    assert!(!directory.path().join("bad").exists());
}
#[test]
fn nonzero_exit_is_recorded_without_retry() {
    let (directory, repository, input) = fixture(7);
    let id = prepare(&repository, input);
    run_job_at(&repository, &id).unwrap();
    let result = ExecutionService::result(&repository, &id).unwrap().unwrap();
    assert_eq!(result.exit_code, Some(7));
    assert!(result.error.is_some());
    assert!(run_job_at(&repository, &id).is_err());
    assert!(directory.path().join("received.txt").exists());
}
#[test]
fn denied_cli_exit_is_not_completed() {
    let (_directory, _repository, input) = fixture(13);
    assert_eq!(execute(&input).unwrap(), Some(13));
}
#[test]
fn tampered_ticket_never_runs() {
    let (directory, repository, input) = fixture(0);
    for field in ["signature", "payload"] {
        let id = prepare(&repository, input.clone());
        let path = repository.run_path(&id, "ticket").unwrap();
        let mut data: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        data[field] = "00".into();
        atomic(&path, &data).unwrap();
        assert!(run_job_at(&repository, &id).is_err());
        assert!(!directory.path().join("received.txt").exists());
    }
}
