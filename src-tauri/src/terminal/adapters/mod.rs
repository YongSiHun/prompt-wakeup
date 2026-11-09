use super::*;
#[cfg(windows)]mod windows;
#[cfg(target_os="macos")]mod macos;
pub fn for_kind(kind:&str)->Result<Box<dyn TerminalAdapter>> {
    #[cfg(windows)] {match kind {"cmd"=>Ok(Box::new(windows::CmdAdapter)),"powershell"|"pwsh"=>Ok(Box::new(windows::PowerShellAdapter)),"wt"=>Ok(Box::new(windows::WindowsTerminalAdapter)),"git_bash"=>Ok(Box::new(windows::GitBashAdapter)),_=>Err("지원하지 않는 Windows 터미널".into())}}
    #[cfg(target_os="macos")] {match kind {"terminal"=>Ok(Box::new(macos::MacTerminalAdapter)),"iterm"=>Ok(Box::new(macos::ITermAdapter)),_=>Err("지원하지 않는 macOS 터미널".into())}}
    #[cfg(not(any(windows,target_os="macos")))] {let _=kind;Err("지원하지 않는 운영체제".into())}
}
