use super::*;
use crate::security::{ps_quote, require, sh_quote};
use std::{os::windows::process::CommandExt, process::Command};

pub struct CmdAdapter;
pub struct PowerShellAdapter;
pub struct WindowsTerminalAdapter;
pub struct GitBashAdapter;
fn spawn(mut command: Command) -> Result<()> {
    command.creation_flags(0x00000010); // CREATE_NEW_CONSOLE: the user's real console.
    command.spawn().map_err(crate::storage::err)?;
    Ok(())
}
fn cmd_line(runner: &Path, id: &str) -> Result<String> {
    let path = runner.to_string_lossy();
    require(!path.contains(['%', '!', '&', '|', '<', '>', '^', '"']), "앱 설치 경로에 cmd가 안전하게 처리할 수 없는 문자가 있습니다. 다른 설치 폴더 또는 PowerShell을 사용하세요.")?;
    Ok(format!("\"\"{path}\" --run-job {id}\""))
}
impl TerminalAdapter for CmdAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        let mut command = Command::new(terminal.executable_path.replace('/', "\\"));
        command.args(["/d", "/s", "/c"]).raw_arg(cmd_line(runner, id)?);
        spawn(command)
    }
}
impl TerminalAdapter for PowerShellAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        let mut command = Command::new(&terminal.executable_path);
        let path = ps_quote(&runner.to_string_lossy());
        command.args(["-NoLogo", "-NoProfile", "-Command"])
            .arg(format!("Start-Process -FilePath {path} -ArgumentList '--run-job','{id}' -NoNewWindow -Wait"));
        spawn(command)
    }
}
impl TerminalAdapter for WindowsTerminalAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        let mut command = Command::new(&terminal.executable_path);
        command.args(["-w", "new", "new-tab"]).arg(Path::new(&system).join("System32").join("cmd.exe"))
            .args(["/d", "/s", "/c"]).raw_arg(cmd_line(runner, id)?);
        spawn(command)
    }
}
impl TerminalAdapter for GitBashAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        let path = runner.to_string_lossy().replace('\\', "/");
        let mut command = Command::new(&terminal.executable_path);
        command.args(["--no-cd", "-c"]).arg(format!("{} --run-job {id}", sh_quote(&path)));
        spawn(command)
    }
}
