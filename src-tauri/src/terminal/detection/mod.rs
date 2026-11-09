use crate::models::TerminalInfo;
use super::info;
pub struct TerminalDetector;
impl TerminalDetector {pub fn detect()->Vec<TerminalInfo>{let mut out=vec![];
#[cfg(windows)]{let system=std::env::var("SystemRoot").unwrap_or_else(|_|"C:\\Windows".into());for (kind,name,path) in [("cmd","Command Prompt",format!("{system}/System32/cmd.exe")),("powershell","Windows PowerShell",format!("{system}/System32/WindowsPowerShell/v1.0/powershell.exe")),("pwsh","PowerShell 7",crate::ai_cli::find("pwsh").unwrap_or_else(||"C:/Program Files/PowerShell/7/pwsh.exe".into())),("wt","Windows Terminal (cmd)",crate::ai_cli::find("wt").unwrap_or_default()),("git_bash","Git Bash", "C:/Program Files/Git/git-bash.exe".into())]{out.push(info(kind,name,path,true));}}
#[cfg(target_os="macos")]{for (kind,name,bundle) in [("terminal","Terminal.app","Terminal.app"),("iterm","iTerm2","iTerm.app"),("warp","Warp","Warp.app")]{let mut p=format!("/Applications/{bundle}");for base in ["/System/Applications/Utilities".to_string(),dirs::home_dir().unwrap_or_default().join("Applications").to_string_lossy().into()]{let candidate=format!("{base}/{bundle}");if std::path::Path::new(&candidate).exists(){p=candidate}}out.push(info(kind,name,p,true));}}
out}}
