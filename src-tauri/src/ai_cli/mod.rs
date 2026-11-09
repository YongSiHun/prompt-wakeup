pub mod adapters;
use adapters::{ClaudeAdapter, CodexAdapter, GeminiAdapter};
use crate::{models::*, security::{require, SecurityValidator}, storage::{err, Result}};
use std::{io::Read, path::{Path, PathBuf}, process::{Child, Command, ExitStatus, Stdio}, time::{Duration, Instant}};

pub trait AiCliAdapter: Send + Sync {
    fn detect(&self) -> Option<String>;
    fn validate(&self, path: &str) -> Result<()>;
    fn get_version(&self, path: &str) -> Result<String>;
    fn build_launch_plan(&self, input: &ScheduleInput) -> LaunchPlan;
    fn supports_exit_tracking(&self) -> bool { true }
}
pub fn adapter(kind: CliKind) -> &'static dyn AiCliAdapter {
    match kind { CliKind::Codex => &CodexAdapter, CliKind::Claude => &ClaudeAdapter, CliKind::Gemini => &GeminiAdapter }
}
pub fn find(name: &str) -> Option<String> {
    let path = which::which(name).ok().or_else(|| fallback_path(name));
    path.map(|path| path.to_string_lossy().into_owned())
}
fn fallback_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let suffixes = [".exe", ".cmd", ".bat"];
    #[cfg(not(windows))]
    let suffixes = [""];
    search_directories().into_iter().flat_map(|base| suffixes.map(|suffix| base.join(format!("{name}{suffix}")))).find(|path| path.is_file())
}
fn search_directories() -> Vec<PathBuf> {
    let mut paths: Vec<_> = dirs::home_dir().into_iter().flat_map(|home| [home.join(".local/bin"), home.join(".npm-global/bin"), home.join(".volta/bin")]).collect();
    paths.extend(platform_directories());
    paths
}
#[cfg(windows)]
fn platform_directories() -> Vec<PathBuf> { std::env::var_os("APPDATA").map(PathBuf::from).map(|path| path.join("npm")).into_iter().collect() }
#[cfg(target_os = "macos")]
fn platform_directories() -> Vec<PathBuf> { ["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from).to_vec() }
#[cfg(not(any(windows, target_os = "macos")))]
fn platform_directories() -> Vec<PathBuf> { Vec::new() }
pub fn validate_identity(kind: CliKind, path: &str) -> Result<()> {
    SecurityValidator::executable(path)?;
    let stem = Path::new(path).file_stem().and_then(|value| value.to_str()).unwrap_or("");
    require(stem.eq_ignore_ascii_case(kind.name()), &format!("{} 실행 파일을 선택하세요.", kind.name()))
}
pub fn command(path: &str, arguments: &[String]) -> Result<Command> {
    SecurityValidator::executable(path)?;
    #[cfg(windows)]
    if crate::security::is_batch(path) { return batch_command(path, arguments); }
    let mut command = Command::new(path);
    command.args(arguments);
    Ok(command)
}
#[cfg(windows)]
fn batch_command(path: &str, arguments: &[String]) -> Result<Command> {
    use std::os::windows::process::CommandExt;
    require(!path.contains(['%', '!', '&', '|', '<', '>', '^', '"']), "셸 런처 경로의 특수 문자는 지원하지 않습니다. native CLI를 선택하세요.")?;
    require(arguments.iter().all(|value| value.chars().all(safe_argument)), "지원하지 않는 셸 런처 인자입니다.")?;
    let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let quoted = arguments.iter().map(|value| format!("\"{value}\"")).collect::<Vec<_>>().join(" ");
    let mut command = Command::new(Path::new(&system).join("System32/cmd.exe"));
    command.args(["/d", "/s", "/c"]).raw_arg(format!("\"\"{path}\" {quoted}\""));
    Ok(command)
}
#[cfg(windows)]
fn safe_argument(character: char) -> bool { character.is_ascii_alphanumeric() || " -_".contains(character) }
pub fn version(path: &str) -> Result<String> {
    let mut command = command(path, &["--version".into()])?;
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    hide_probe(&mut command);
    let mut child = command.spawn().map_err(err)?;
    let status = wait_for_version(&mut child)?;
    require(status.success(), "CLI 버전 검사 실패. 터미널에서 설치 및 로그인을 확인하세요.")?;
    let stdout = child.stdout.take().ok_or("CLI 출력 채널이 없습니다.")?;
    let mut output = String::new();
    stdout.take(4096).read_to_string(&mut output).map_err(err)?;
    Ok(output.trim().into())
}
fn wait_for_version(child: &mut Child) -> Result<ExitStatus> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(err)? { return Ok(status); }
        if start.elapsed() >= Duration::from_secs(5) { return version_timeout(child); }
        std::thread::sleep(Duration::from_millis(40));
    }
}
fn version_timeout(child: &mut Child) -> Result<ExitStatus> {
    let _ = child.kill();
    let _ = child.wait();
    Err("CLI 버전 확인 시간 초과".into())
}
#[cfg(windows)]
fn hide_probe(command: &mut Command) { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
#[cfg(not(windows))]
fn hide_probe(_: &mut Command) {}
pub fn inspect(kind: CliKind, path: String) -> CliInfo {
    let adapter = adapter(kind);
    let result = adapter.validate(&path).and_then(|_| adapter.get_version(&path));
    CliInfo { kind, executable_path: path, available: result.is_ok(), version: result.as_ref().ok().cloned(), validation_error: result.err() }
}
pub fn detect(settings: &Settings) -> Vec<CliInfo> {
    std::thread::scope(|scope| {
        let workers: Vec<_> = CliKind::ALL.into_iter().map(|kind| scope.spawn(move || detect_one(kind, settings))).collect();
        workers.into_iter().zip(CliKind::ALL).map(|(worker, kind)| worker.join().unwrap_or_else(|_| detection_failure(kind))).collect()
    })
}
fn detect_one(kind: CliKind, settings: &Settings) -> CliInfo {
    let path = settings.custom_cli_paths.get(kind.name()).cloned().or_else(|| adapter(kind).detect()).unwrap_or_default();
    inspect(kind, path)
}
fn detection_failure(kind: CliKind) -> CliInfo {
    CliInfo { kind, executable_path: String::new(), available: false, version: None, validation_error: Some("CLI 탐지 작업이 중단되었습니다. 다시 탐색하세요.".into()) }
}
pub fn plan(input: &ScheduleInput, arguments: Vec<String>) -> LaunchPlan {
    LaunchPlan {
        executable: input.cli_path.clone(), arguments, working_directory: input.working_directory.clone(), environment: Default::default(),
        terminal_adapter_id: input.terminal.kind.clone(), prompt_delivery: "stdin_utf8".into(), status_tracking_strategy: "signed_ticket_result_file".into(),
        close_behavior: if input.keep_open { "wait_for_enter" } else { "exit" }.into(),
    }
}
