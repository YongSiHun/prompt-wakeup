use super::*;
// Test: fixture.
fn fixture(code: i32) -> (tempfile::TempDir, StorageRepository, ScheduleInput) {
    let directory = tempfile::tempdir().unwrap();
    let repository = StorageRepository::at(directory.path().join("data")).unwrap();
    let input = crate::test_support::input(directory.path());
    let script = format!("const fs=require('node:fs');if(process.argv.includes('--version')){{console.log('fixture 1');process.exit(0)}}console.log('한글 stdout');console.error('stderr fixture');const data=[];process.stdin.on('data',c=>data.push(c));process.stdin.on('end',()=>{{fs.writeFileSync('received.txt',Buffer.concat(data));process.exit({code})}});");
    std::fs::write(directory.path().join("fixture.cjs"), script).unwrap();
    std::fs::write(&input.cli_path, cli_shim()).unwrap();
    (directory, repository, input)
}
// Test: cli shim.
fn cli_shim() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\nnode \"%~dp0fixture.cjs\" %*\r\n"
    } else {
        "#!/bin/sh\nexec node \"$(dirname \"$0\")/fixture.cjs\" \"$@\"\n"
    }
}
// Test: prepare.
fn prepare(repository: &StorageRepository, input: ScheduleInput) -> String {
    let schedule = Schedule {
        id: uuid::Uuid::new_v4().to_string(),
        input,
        status: ScheduleStatus::Pending,
    };
    let id = uuid::Uuid::new_v4().to_string();
    ExecutionService::prepare(repository, &schedule, &id).unwrap();
    id
}
#[test]
// Test: noninteractive stdin preserves unicode quotes newlines.
fn noninteractive_stdin_preserves_unicode_quotes_newlines() {
    let (directory, _repository, input) = fixture(0);
    assert_eq!(
        execute(&input, &tempfile::tempfile().unwrap()).unwrap(),
        Some(0)
    );
    assert_eq!(
        std::fs::read_to_string(directory.path().join("received.txt")).unwrap(),
        input.prompt
    );
    assert!(!directory.path().join("bad").exists());
}
#[test]
// Test: nonzero exit is recorded without retry.
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
// Test: denied cli exit is not completed.
fn denied_cli_exit_is_not_completed() {
    let (_directory, _repository, input) = fixture(13);
    assert_eq!(
        execute(&input, &tempfile::tempfile().unwrap()).unwrap(),
        Some(13)
    );
}
#[test]
// Test: tampered ticket never runs.
fn tampered_ticket_never_runs() {
    let (directory, repository, input) = fixture(0);
    for field in ["signature", "payload"] {
        let id = prepare(&repository, input.clone());
        let path = repository.run_path(&id, "ticket").unwrap();
        let mut data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        data[field] = "00".into();
        atomic(&path, &data).unwrap();
        assert!(run_job_at(&repository, &id).is_err());
        assert!(!directory.path().join("received.txt").exists());
    }
}

#[test]
// Test: output survives completion and repository reopen.
fn output_survives_completion_and_repository_reopen() {
    let (_directory, repository, input) = fixture(0);
    let id = prepare(&repository, input);
    run_job_at(&repository, &id).unwrap();
    let reopened = StorageRepository::at(repository.root.clone()).unwrap();
    let text = std::fs::read_to_string(reopened.run_path(&id, "output").unwrap()).unwrap();
    assert!(text.contains("한글 stdout"));
    assert!(text.contains("stderr fixture"));
    assert!(text.contains("정상 종료"));
    reopened.remove_run(&id);
    assert!(!reopened.run_path(&id, "output").unwrap().exists());
}

#[test]
// Test: large stdout and stderr are not truncated.
fn large_stdout_and_stderr_are_not_truncated() {
    let (directory, repository, input) = fixture(7);
    std::fs::write(directory.path().join("fixture.cjs"),
        "process.stdin.resume();process.stdin.on('end',()=>{require('fs').writeSync(1,'x'.repeat(200000));require('fs').writeSync(2,'y'.repeat(200000));process.exit(7)});").unwrap();
    let id = prepare(&repository, input);
    run_job_at(&repository, &id).unwrap();
    let text = std::fs::read_to_string(repository.run_path(&id, "output").unwrap()).unwrap();
    assert_eq!(text.matches('x').count(), 200000);
    assert_eq!(text.matches('y').count(), 200000);
    assert_eq!(
        ExecutionService::result(&repository, &id)
            .unwrap()
            .unwrap()
            .exit_code,
        Some(7)
    );
}
