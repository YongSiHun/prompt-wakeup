#[cfg(target_os = "macos")]
mod environment;
mod version;
use crate::{
    models::*,
    security::{require, SecurityValidator},
    storage::Result,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
pub use version::version;

// CLI: 실행 파일 경로 탐색.
pub fn find(name: &str) -> Option<String> {
    find_in_path(name)
        .ok()
        .or_else(|| fallback_path(name))
        .map(|path| path.to_string_lossy().into_owned())
}
// CLI: Use the same search path for detection and execution.
fn find_in_path(name: &str) -> std::result::Result<PathBuf, which::Error> {
    #[cfg(target_os = "macos")]
    return which::which_in(name, Some(environment::path()), "/");
    #[cfg(not(target_os = "macos"))]
    which::which(name)
}
fn fallback_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let suffixes = [".exe", ".cmd", ".bat"];
    #[cfg(not(windows))]
    let suffixes = [""];
    search_directories()
        .into_iter()
        .flat_map(|base| suffixes.map(|suffix| base.join(format!("{name}{suffix}"))))
        .find(|path| SecurityValidator::executable(&path.to_string_lossy()).is_ok())
}
// CLI: 사용자 설치 경로.
fn search_directories() -> Vec<PathBuf> {
    let mut paths: Vec<_> = dirs::home_dir()
        .into_iter()
        .flat_map(|home| {
            [
                home.join(".local/bin"),
                home.join(".npm-global/bin"),
                home.join(".volta/bin"),
            ]
        })
        .collect();
    paths.extend(platform_directories());
    paths
}
#[cfg(windows)]
// CLI: OS별 설치 경로.
fn platform_directories() -> Vec<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("npm"))
        .into_iter()
        .collect()
}
#[cfg(target_os = "macos")]
// CLI: OS별 설치 경로.
fn platform_directories() -> Vec<PathBuf> {
    ["/opt/homebrew/bin", "/usr/local/bin"]
        .map(PathBuf::from)
        .to_vec()
}
#[cfg(not(any(windows, target_os = "macos")))]
// CLI: OS별 설치 경로.
fn platform_directories() -> Vec<PathBuf> {
    Vec::new()
}
// 검증: CLI 파일·이름.
pub fn validate_identity(kind: CliKind, path: &str) -> Result<()> {
    SecurityValidator::executable(path)?;
    let stem = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    require(
        stem.eq_ignore_ascii_case(kind.name()),
        &format!("{} 실행 파일을 선택하세요.", kind.name()),
    )
}
// CLI: 안전한 명령 구성.
pub fn command(path: &str, arguments: &[String]) -> Result<Command> {
    SecurityValidator::executable(path)?;
    #[cfg(windows)]
    if crate::security::is_batch(path) {
        return batch_command(path, arguments);
    }
    let mut command = Command::new(path);
    command.args(arguments);
    #[cfg(target_os = "macos")]
    environment::configure(&mut command, Path::new(path))?;
    Ok(command)
}
#[cfg(windows)]
// OS: 검증된 batch 명령 구성.
fn batch_command(path: &str, arguments: &[String]) -> Result<Command> {
    use std::os::windows::process::CommandExt;
    require(
        !path.contains(['%', '!', '&', '|', '<', '>', '^', '"']),
        "셸 런처 경로의 특수 문자는 지원하지 않습니다. native CLI를 선택하세요.",
    )?;
    require(
        arguments
            .iter()
            .all(|value| value.chars().all(safe_argument)),
        "지원하지 않는 셸 런처 인자입니다.",
    )?;
    let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let quoted = arguments
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(" ");
    let mut command = Command::new(Path::new(&system).join("System32").join("cmd.exe"));
    command
        .args(["/d", "/s", "/c"])
        .raw_arg(format!("\"\"{path}\" {quoted}\""));
    Ok(command)
}
#[cfg(windows)]
// 검증: 셸 인자 허용 문자.
fn safe_argument(character: char) -> bool {
    character.is_ascii_alphanumeric() || " -_".contains(character)
}
// CLI: 파일·버전 검사.
pub fn inspect(kind: CliKind, path: String) -> CliInfo {
    let result = validate_identity(kind, &path).and_then(|_| version(&path));
    CliInfo {
        kind,
        executable_path: path,
        available: result.is_ok(),
        version: result.as_ref().ok().cloned(),
        validation_error: result.err(),
    }
}
// CLI: 설치 상태 병렬 조회.
pub fn detect(settings: &Settings) -> Vec<CliInfo> {
    std::thread::scope(|scope| {
        let workers: Vec<_> = CliKind::ALL
            .into_iter()
            .map(|kind| scope.spawn(move || detect_one(kind, settings)))
            .collect();
        workers
            .into_iter()
            .zip(CliKind::ALL)
            .map(|(worker, kind)| worker.join().unwrap_or_else(|_| detection_failure(kind)))
            .collect()
    })
}
// CLI: 사용자 설정 우선 탐지.
fn detect_one(kind: CliKind, settings: &Settings) -> CliInfo {
    let path = settings
        .custom_cli_paths
        .get(kind.name())
        .cloned()
        .or_else(|| find(kind.name()))
        .unwrap_or_default();
    inspect(kind, path)
}
// CLI: 탐지 실패 결과.
fn detection_failure(kind: CliKind) -> CliInfo {
    CliInfo {
        kind,
        executable_path: String::new(),
        available: false,
        version: None,
        validation_error: Some("CLI 탐지 작업이 중단되었습니다. 다시 탐색하세요.".into()),
    }
}
// CLI: 종류별 실행 인자 구성.
pub fn plan(input: &ScheduleInput) -> LaunchPlan {
    let arguments = match input.cli {
        CliKind::Codex => vec!["exec".into(), "-".into()],
        CliKind::Claude => vec!["-p".into()],
        CliKind::Gemini => vec!["-p".into(), "Execute the task supplied on stdin".into()],
    };
    LaunchPlan {
        executable: input.cli_path.clone(),
        arguments,
        working_directory: input.working_directory.clone(),
    }
}

#[cfg(test)]
mod tests;
