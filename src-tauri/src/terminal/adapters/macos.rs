use super::*;
use crate::security::sh_quote;
use std::process::Command;
pub struct MacTerminalAdapter;
pub struct ITermAdapter;
fn launch_script(t:&TerminalInfo,runner:&Path,id:&str,script:&str)->Result<()> {let command=format!("{} --run-job {id}; exit",sh_quote(&runner.to_string_lossy()));let output=Command::new("/usr/bin/osascript").args(["-e",script,"--",&t.executable_path,&command]).output().map_err(crate::storage::err)?;if !output.status.success(){return Err(format!("터미널 자동화 실패. 시스템 설정 > 개인정보 보호 및 보안 > 자동화 권한을 확인하세요. {}",String::from_utf8_lossy(&output.stderr)))}Ok(())}
impl TerminalAdapter for MacTerminalAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {launch_script(t,runner,id,"on run argv\nusing terms from application \"Terminal\"\ntell application (item 1 of argv)\nactivate\ndo script (item 2 of argv)\nend tell\nend using terms from\nend run")}}
impl TerminalAdapter for ITermAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {launch_script(t,runner,id,"on run argv\nusing terms from application \"iTerm\"\ntell application (item 1 of argv)\nactivate\ncreate window with default profile command (item 2 of argv)\nend tell\nend using terms from\nend run")}}
