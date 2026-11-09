use super::*;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
pub fn for_kind(kind: &str) -> Result<&'static dyn TerminalAdapter> {
    match kind {
        "cmd" => Ok(&windows::CmdAdapter), "powershell" | "pwsh" => Ok(&windows::PowerShellAdapter),
        "wt" => Ok(&windows::WindowsTerminalAdapter), "git_bash" => Ok(&windows::GitBashAdapter),
        _ => Err("알려진 Windows 터미널 어댑터와 연결할 수 없습니다.".into()),
    }
}
#[cfg(target_os = "macos")]
pub fn for_kind(kind: &str) -> Result<&'static dyn TerminalAdapter> {
    match kind { "terminal" => Ok(&macos::MacTerminalAdapter), "iterm" => Ok(&macos::ITermAdapter), _ => Err("알려진 macOS 터미널 어댑터와 연결할 수 없습니다.".into()) }
}
#[cfg(not(any(windows, target_os = "macos")))]
pub fn for_kind(_: &str) -> Result<&'static dyn TerminalAdapter> { Err("지원하지 않는 운영체제".into()) }
