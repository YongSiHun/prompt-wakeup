use crate::{execution::{ExecutionService, process_probe}, models::*, storage::StorageRepository};
use chrono::{DateTime, Duration, Utc};
use std::fs;

pub fn observe(repository: &StorageRepository, execution: &ExecutionRecord, now: DateTime<Utc>, probe: bool) -> Option<ExecutionRecord> {
    match ExecutionService::result(repository, &execution.id) {
        Ok(Some(result)) => Some(completed(execution, result)),
        Err(error) => Some(unknown(execution, now, format!("실행 결과 파일을 확인할 수 없습니다: {error}"))),
        Ok(None) => probe_claim(repository, execution, now, probe),
    }
}
fn completed(original: &ExecutionRecord, result: RunResult) -> ExecutionRecord {
    let mut execution = original.clone();
    let success = result.exit_code == Some(0) && result.error.is_none();
    execution.status = if success { ExecutionStatus::Completed } else { ExecutionStatus::Failed };
    execution.exit_code = result.exit_code;
    execution.ended_at = Some(result.ended_at);
    execution.error = result.error;
    execution
}
fn unknown(original: &ExecutionRecord, now: DateTime<Utc>, error: String) -> ExecutionRecord {
    let mut execution = original.clone();
    execution.mark_unknown(now, error);
    execution
}
fn probe_claim(repository: &StorageRepository, execution: &ExecutionRecord, now: DateTime<Utc>, probe: bool) -> Option<ExecutionRecord> {
    if !probe { return None; }
    let path = repository.run_path(&execution.id, "claim").ok()?;
    let claim = fs::read_to_string(&path);
    let pid = claim.as_ref().ok().and_then(|text| text.trim().parse::<u32>().ok());
    match pid {
        Some(pid) => lost_process(execution, now, pid),
        None => expired_claim(repository, execution, now, !path.exists() || claim.is_ok()),
    }
}
fn lost_process(execution: &ExecutionRecord, now: DateTime<Utc>, pid: u32) -> Option<ExecutionRecord> {
    (!process_probe::alive(pid)).then(|| unknown(execution, now, "실행 래퍼가 결과를 기록하지 않고 종료되었습니다. 외부 터미널과 CLI가 중단되었는지 확인한 후 수동 재실행하세요.".into()))
}
fn expired_claim(repository: &StorageRepository, execution: &ExecutionRecord, now: DateTime<Utc>, missing: bool) -> Option<ExecutionRecord> {
    if !missing || now <= execution.started_at + Duration::minutes(10) { return None; }
    repository.remove_run_file(&execution.id, "ticket");
    Some(unknown(execution, now, "실행 래퍼 시작을 확인할 수 없습니다. 외부 터미널을 확인한 후 수동 재실행하세요.".into()))
}
