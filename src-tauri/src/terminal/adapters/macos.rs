use super::*;
use crate::{
    security::{require, sh_quote},
    storage::err,
};
use std::process::Command;

pub struct MacTerminalAdapter;
pub struct ITermAdapter;
const TERMINAL_SCRIPT: &str = r#"on run argv
using terms from application "Terminal"
tell application (item 1 of argv)
activate
do script (item 2 of argv)
end tell
end using terms from
end run"#;
const ITERM_SCRIPT: &str = r#"on run argv
using terms from application "iTerm"
tell application (item 1 of argv)
activate
create window with default profile command (item 2 of argv)
end tell
end using terms from
end run"#;
fn launch_script(terminal: &TerminalInfo, runner: &Path, id: &str, script: &str) -> Result<()> {
    let command = format!(
        "{} --run-job {id}; exit",
        sh_quote(&runner.to_string_lossy())
    );
    let output = Command::new("/usr/bin/osascript")
        .args(["-e", script, "--", &terminal.executable_path, &command])
        .output()
        .map_err(err)?;
    require(output.status.success(), &format!("터미널 자동화 실패. 시스템 설정 > 개인정보 보호 및 보안 > 자동화 권한을 확인하세요. {}", String::from_utf8_lossy(&output.stderr)))
}
impl TerminalAdapter for MacTerminalAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        launch_script(terminal, runner, id, TERMINAL_SCRIPT)
    }
}
impl TerminalAdapter for ITermAdapter {
    fn launch(&self, terminal: &TerminalInfo, runner: &Path, id: &str) -> Result<()> {
        launch_script(terminal, runner, id, ITERM_SCRIPT)
    }
}
