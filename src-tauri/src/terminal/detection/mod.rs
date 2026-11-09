use crate::{models::{Settings, TerminalInfo}, terminal::info};
use std::{collections::BTreeMap, path::PathBuf};
pub struct TerminalDetector;
impl TerminalDetector {
    pub fn detect() -> Vec<TerminalInfo> { platform_terminals() }
    pub fn configured(settings: &Settings) -> Vec<TerminalInfo> {
        let custom = settings.custom_terminals.iter().filter_map(|terminal| super::manual(terminal.executable_path.clone()).ok());
        Self::detect().into_iter().chain(custom).map(|terminal| (terminal.id.clone(), terminal)).collect::<BTreeMap<_, _>>().into_values().collect()
    }
}
#[cfg(windows)]
fn platform_terminals() -> Vec<TerminalInfo> {
    let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    [
        ("cmd", "Command Prompt", PathBuf::from(&system).join("System32").join("cmd.exe").to_string_lossy().into_owned()),
        ("powershell", "Windows PowerShell", PathBuf::from(&system).join("System32").join("WindowsPowerShell").join("v1.0").join("powershell.exe").to_string_lossy().into_owned()),
        ("pwsh", "PowerShell 7", crate::ai_cli::find("pwsh").unwrap_or_else(|| "C:/Program Files/PowerShell/7/pwsh.exe".into())),
        ("wt", "Windows Terminal (cmd)", crate::ai_cli::find("wt").unwrap_or_default()),
        ("git_bash", "Git Bash", git_bash_path()),
    ].into_iter().map(|(kind, name, path)| info(kind, name, path, true)).collect()
}
#[cfg(windows)]
fn git_bash_path() -> String {
    let bases = [std::env::var_os("ProgramFiles").map(PathBuf::from), std::env::var_os("LOCALAPPDATA").map(|path| PathBuf::from(path).join("Programs"))];
    bases.into_iter().flatten().map(|base| base.join("Git/git-bash.exe")).find(|path| path.is_file()).unwrap_or_else(|| PathBuf::from("C:/Program Files/Git/git-bash.exe")).to_string_lossy().into()
}
#[cfg(target_os = "macos")]
fn platform_terminals() -> Vec<TerminalInfo> {
    [("terminal", "Terminal.app", "Terminal.app"), ("iterm", "iTerm2", "iTerm.app"), ("warp", "Warp", "Warp.app")]
        .into_iter().map(|(kind, name, bundle)| info(kind, name, mac_bundle(bundle), true)).collect()
}
#[cfg(target_os = "macos")]
fn mac_bundle(bundle: &str) -> String {
    let home = dirs::home_dir().unwrap_or_default().join("Applications");
    [PathBuf::from("/Applications"), PathBuf::from("/System/Applications/Utilities"), home]
        .into_iter().map(|base| base.join(bundle)).find(|path| path.is_dir()).unwrap_or_else(|| PathBuf::from("/Applications").join(bundle)).to_string_lossy().into()
}
#[cfg(not(any(windows, target_os = "macos")))]
fn platform_terminals() -> Vec<TerminalInfo> { Vec::new() }
