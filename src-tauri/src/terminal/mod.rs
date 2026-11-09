pub mod adapters;
pub mod detection;
#[cfg(windows)]
use crate::security::SecurityValidator;
use crate::{
    models::TerminalInfo,
    security::require,
    storage::{err, Result},
};
use std::path::Path;

pub fn info(kind: &str, name: &str, path: String, automatic: bool) -> TerminalInfo {
    let installed = Path::new(&path).exists();
    let validation_error = terminal_error(kind, installed);
    TerminalInfo {
        id: format!("{kind}:{path}"),
        name: name.into(),
        kind: kind.into(),
        executable_path: path,
        detected_automatically: automatic,
        installed,
        supported: validation_error.is_none(),
        validation_error,
    }
}
fn terminal_error(kind: &str, installed: bool) -> Option<String> {
    match (kind, installed) {
        ("warp", _) => Some("Warp의 검증된 임의 작업 실행 어댑터는 아직 지원하지 않습니다.".into()),
        (_, false) => Some("설치되지 않았습니다.".into()),
        _ => adapters::for_kind(kind).err(),
    }
}
pub fn manual(path: String) -> Result<TerminalInfo> {
    let name = Path::new(&path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    Ok(info(kind_from_name(&name), &name, path, false))
}
fn kind_from_name(name: &str) -> &str {
    match name {
        "cmd" | "powershell" | "pwsh" | "wt" | "terminal" | "warp" => name,
        "git-bash" => "git_bash",
        "iterm" | "iterm2" => "iterm",
        _ => "unsupported",
    }
}
pub fn validate(terminal: &TerminalInfo) -> Result<()> {
    let recognized = manual(terminal.executable_path.clone())?;
    require(
        recognized.supported && recognized.kind == terminal.kind,
        "지원되는 터미널을 다시 선택하세요.",
    )?;
    validate_platform(terminal)
}
#[cfg(windows)]
fn validate_platform(terminal: &TerminalInfo) -> Result<()> {
    SecurityValidator::executable(&terminal.executable_path).map(|_| ())
}
#[cfg(target_os = "macos")]
fn validate_platform(terminal: &TerminalInfo) -> Result<()> {
    require(
        Path::new(&terminal.executable_path).is_dir(),
        "지원되는 macOS .app을 선택하세요.",
    )
}
#[cfg(not(any(windows, target_os = "macos")))]
fn validate_platform(_: &TerminalInfo) -> Result<()> {
    Err("지원하지 않는 운영체제입니다.".into())
}
pub trait TerminalAdapter: Send + Sync {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()>;
}
pub struct TerminalLauncher;
impl TerminalLauncher {
    pub fn launch(terminal: &TerminalInfo, id: &str) -> Result<()> {
        validate(terminal)?;
        uuid::Uuid::parse_str(id).map_err(|_| "실행 ID 오류")?;
        let runner = std::env::current_exe().map_err(err)?;
        adapters::for_kind(&terminal.kind)?.launch(terminal, &runner, id)
    }
}
