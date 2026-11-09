#[cfg(unix)]
use crate::storage::err;
use crate::{models::ScheduleInput, storage::Result};
use chrono::{LocalResult, NaiveDateTime, TimeZone, Utc};
use std::path::{Path, PathBuf};

pub struct SecurityValidator;
impl SecurityValidator {
    pub fn executable(path: &str) -> Result<PathBuf> {
        require(
            !path.contains(['\0', '\n', '\r']),
            "실행 경로에 제어 문자를 사용할 수 없습니다.",
        )?;
        let path = Path::new(path);
        require(
            path.is_absolute() && path.is_file(),
            "실행 파일이 없습니다. 다시 탐색하거나 경로를 선택하세요.",
        )?;
        executable_permission(path)?;
        Ok(path.to_path_buf())
    }
    pub fn input(input: &ScheduleInput, future: bool) -> Result<()> {
        validate_content(input)?;
        validate_directory(input)?;
        crate::ai_cli::validate_identity(input.cli, &input.cli_path)?;
        crate::terminal::validate(&input.terminal)?;
        require(
            !future || input.scheduled_at > Utc::now(),
            "미래 실행 시각을 입력하세요.",
        )?;
        validate_time(input)
    }
}

pub fn require(valid: bool, message: &str) -> Result<()> {
    valid.then_some(()).ok_or_else(|| message.to_owned())
}
fn validate_content(input: &ScheduleInput) -> Result<()> {
    require(
        !input.name.trim().is_empty() && input.name.len() <= 300,
        "작업 이름을 1~300바이트로 입력하세요.",
    )?;
    let valid = !input.prompt.trim().is_empty()
        && input.prompt.len() <= 1_000_000
        && !input.prompt.contains('\0');
    require(
        valid,
        "프롬프트는 필수이며 1MB 이하, NUL 문자를 포함할 수 없습니다.",
    )
}
fn validate_directory(input: &ScheduleInput) -> Result<()> {
    let path = Path::new(&input.working_directory);
    require(
        path.is_absolute() && path.is_dir(),
        "작업 폴더가 없습니다. 존재하는 절대 경로를 선택하세요.",
    )?;
    validate_batch_directory(input)
}
#[cfg(windows)]
fn validate_batch_directory(input: &ScheduleInput) -> Result<()> {
    let unc =
        input.working_directory.starts_with("\\\\") || input.working_directory.starts_with("//");
    require(!(is_batch(&input.cli_path) && unc), "Windows batch CLI는 UNC 작업 폴더를 지원하지 않습니다. 로컬 폴더 또는 native CLI를 선택하세요.")
}
#[cfg(not(windows))]
fn validate_batch_directory(_: &ScheduleInput) -> Result<()> {
    Ok(())
}
#[cfg(unix)]
fn executable_permission(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    require(
        path.metadata().map_err(err)?.permissions().mode() & 0o111 != 0,
        "실행 권한이 없습니다.",
    )
}
#[cfg(not(unix))]
fn executable_permission(_: &Path) -> Result<()> {
    Ok(())
}
fn validate_time(input: &ScheduleInput) -> Result<()> {
    let timezone: chrono_tz::Tz = input
        .timezone
        .parse()
        .map_err(|_| "IANA 시간대가 올바르지 않습니다.")?;
    let local = NaiveDateTime::parse_from_str(&input.local_time, "%Y-%m-%dT%H:%M:%S")
        .map_err(|_| "초 단위 로컬 시각을 입력하세요.")?;
    match timezone.from_local_datetime(&local) {
        LocalResult::Single(time) => require(
            time.with_timezone(&Utc) == input.scheduled_at,
            "시간대와 UTC 시각이 일치하지 않습니다.",
        ),
        LocalResult::Ambiguous(_, _) => Err("DST 중복 시각입니다. 다른 시각을 선택하세요.".into()),
        LocalResult::None => Err("존재하지 않는 로컬 시각입니다. 다른 시각을 선택하세요.".into()),
    }
}
pub fn is_batch(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    ["cmd", "bat"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}
pub fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
pub fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
