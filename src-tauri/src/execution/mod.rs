pub mod process_probe;
use crate::{models::*, security::{require, SecurityValidator}, storage::{atomic, err, Result, StorageRepository}};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{fs::{self, OpenOptions}, io::Write, process::{ExitStatus, Stdio}};

#[derive(serde::Serialize, serde::Deserialize)]
struct SignedTicket { payload: String, signature: String }
pub struct ExecutionService;
impl ExecutionService {
    pub fn prepare(repository: &StorageRepository, schedule: &Schedule, id: &str) -> Result<()> {
        SecurityValidator::input(&schedule.input, false)?;
        let ticket = Ticket { execution_id: id.into(), schedule: schedule.input.clone(), expires_at: Utc::now() + Duration::minutes(10) };
        let payload = serde_json::to_string(&ticket).map_err(err)?;
        let mut mac = new_mac(repository, true)?;
        mac.update(payload.as_bytes());
        let signed = SignedTicket { payload, signature: hex::encode(mac.finalize().into_bytes()) };
        atomic(&repository.run_path(id, "ticket")?, &signed)
    }
    pub fn result(repository: &StorageRepository, id: &str) -> Result<Option<RunResult>> {
        let path = repository.run_path(id, "result")?;
        if !path.exists() { return Ok(None); }
        let result: RunResult = serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(err)?;
        require(result.execution_id == id, "실행 결과 ID 불일치")?;
        Ok(Some(result))
    }
}
fn new_mac(repository: &StorageRepository, create: bool) -> Result<Hmac<Sha256>> {
    Hmac::<Sha256>::new_from_slice(&repository.key(create)?).map_err(err)
}
fn read_ticket(repository: &StorageRepository, id: &str) -> Result<Ticket> {
    let bytes = fs::read(repository.run_path(id, "ticket")?).map_err(err)?;
    let signed: SignedTicket = serde_json::from_slice(&bytes).map_err(err)?;
    let mut mac = new_mac(repository, false)?;
    mac.update(signed.payload.as_bytes());
    mac.verify_slice(&hex::decode(signed.signature).map_err(err)?).map_err(|_| "실행 데이터 무결성 검증 실패")?;
    let ticket: Ticket = serde_json::from_str(&signed.payload).map_err(err)?;
    require(ticket.execution_id == id && ticket.expires_at >= Utc::now(), "만료되었거나 올바르지 않은 실행 요청")?;
    Ok(ticket)
}
fn claim(repository: &StorageRepository, id: &str) -> Result<()> {
    let mut claim = OpenOptions::new().write(true).create_new(true).open(repository.run_path(id, "claim")?).map_err(|_| "이미 시작한 실행 요청입니다.")?;
    write!(claim, "{}", std::process::id()).map_err(err)?;
    claim.sync_all().map_err(err)
}
pub fn run_job(id: &str) -> Result<()> { run_job_at(&StorageRepository::open()?, id) }
fn run_job_at(repository: &StorageRepository, id: &str) -> Result<()> {
    let ticket = read_ticket(repository, id)?;
    claim(repository, id)?;
    let result = execution_result(id, execute(&ticket.schedule));
    atomic(&repository.run_path(id, "result")?, &result)?;
    repository.remove_run_file(id, "ticket");
    show_result(&result, ticket.schedule.keep_open);
    Ok(())
}
fn execution_result(id: &str, outcome: Result<Option<i32>>) -> RunResult {
    match outcome {
        Ok(code) => RunResult { execution_id: id.into(), ended_at: Utc::now(), exit_code: code, error: cli_error(code) },
        Err(error) => RunResult { execution_id: id.into(), ended_at: Utc::now(), exit_code: None, error: Some(error) },
    }
}
fn cli_error(code: Option<i32>) -> Option<String> {
    (code != Some(0)).then(|| "CLI가 실패했습니다. 외부 터미널에서 인증, 사용량 제한, 네트워크 및 권한 승인 정책을 확인하세요. 자동 재시도하지 않습니다.".into())
}
fn show_result(result: &RunResult, keep_open: bool) {
    println!("\nPromptWakeup: 종료 코드 {:?} {}", result.exit_code, result.error.as_deref().unwrap_or("정상 종료"));
    if keep_open { wait_for_enter(); }
}
fn wait_for_enter() {
    println!("창을 닫으려면 Enter를 누르세요.");
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
}
fn execute(input: &ScheduleInput) -> Result<Option<i32>> {
    SecurityValidator::input(input, false)?;
    let plan = crate::ai_cli::adapter(input.cli).build_launch_plan(input);
    let mut command = crate::ai_cli::command(&plan.executable, &plan.arguments)?;
    command.current_dir(&plan.working_directory).stdin(Stdio::piped()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let mut child = command.spawn().map_err(err)?;
    let mut stdin = child.stdin.take().ok_or("CLI stdin 없음")?;
    std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(input.prompt.as_bytes()));
        let status = child.wait().map_err(err)?;
        let delivered = writer.join().map_err(|_| "프롬프트 전달 스레드 오류")?;
        validate_delivery(status, delivered)?;
        Ok(status.code())
    })
}
fn validate_delivery(status: ExitStatus, delivered: std::io::Result<()>) -> Result<()> {
    if status.success() { delivered.map_err(err)?; }
    Ok(())
}
#[cfg(test)]
mod tests;
