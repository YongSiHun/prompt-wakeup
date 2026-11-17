pub mod launcher;
mod process;
pub mod process_probe;
mod ticket;
use crate::{
    models::*,
    security::require,
    storage::{atomic, err, Result, StorageRepository},
};
use chrono::Utc;
use process::execute;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

pub struct ExecutionService;
impl ExecutionService {
    pub fn prepare(repository: &StorageRepository, schedule: &Schedule, id: &str) -> Result<()> {
        ticket::prepare(repository, schedule, id)
    }
    pub fn result(repository: &StorageRepository, id: &str) -> Result<Option<RunResult>> {
        let path = repository.run_path(id, "result")?;
        if !path.exists() {
            return Ok(None);
        }
        let result: RunResult =
            serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(err)?;
        require(result.execution_id == id, "실행 결과 ID 불일치")?;
        Ok(Some(result))
    }
}
pub fn run_job(id: &str) -> Result<()> {
    run_job_at(&StorageRepository::open()?, id)
}
fn run_job_at(repository: &StorageRepository, id: &str) -> Result<()> {
    let ticket = ticket::read(repository, id)?;
    ticket::claim(repository, id)?;
    let output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(repository.run_path(id, "output")?)
        .map_err(err)?;
    let result = execution_result(id, execute(&ticket.schedule, &output));
    writeln!(
        &output,
        "\nPromptWakeup: 종료 코드 {:?} {}",
        result.exit_code,
        result.error.as_deref().unwrap_or("정상 종료")
    )
    .map_err(err)?;
    output.sync_all().map_err(err)?;
    atomic(&repository.run_path(id, "result")?, &result)?;
    repository.remove_run_file(id, "ticket");
    Ok(())
}
fn execution_result(id: &str, outcome: Result<Option<i32>>) -> RunResult {
    match outcome {
        Ok(code) => RunResult {
            execution_id: id.into(),
            ended_at: Utc::now(),
            exit_code: code,
            error: cli_error(code),
        },
        Err(error) => RunResult {
            execution_id: id.into(),
            ended_at: Utc::now(),
            exit_code: None,
            error: Some(error),
        },
    }
}
fn cli_error(code: Option<i32>) -> Option<String> {
    (code != Some(0)).then(|| "CLI가 실패했습니다. 앱 내 터미널에서 인증, 사용량 제한, 네트워크 및 권한 승인 정책을 확인하세요. 자동 재시도하지 않습니다.".into())
}
#[cfg(test)]
mod tests;
