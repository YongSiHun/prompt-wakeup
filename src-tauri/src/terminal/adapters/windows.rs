use super::*;
use crate::security::{ps_quote,sh_quote};
use std::{process::Command,os::windows::process::CommandExt};
pub struct CmdAdapter;
pub struct PowerShellAdapter;
pub struct WindowsTerminalAdapter;
pub struct GitBashAdapter;
fn spawn(mut command:Command)->Result<()> {command.creation_flags(0x00000010);command.spawn().map_err(crate::storage::err)?;Ok(())}
fn cmd_line(runner:&Path,id:&str)->Result<String> {let r=runner.to_string_lossy();if r.contains(['%','!','&','|','<','>','^','"']){return Err("앱 설치 경로에 cmd가 안전하게 처리할 수 없는 문자가 있습니다. 다른 설치 폴더 또는 PowerShell을 사용하세요.".into())}Ok(format!("\"\"{r}\" --run-job {id}\""))}
impl TerminalAdapter for CmdAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {let mut c=Command::new(&t.executable_path);c.args(["/d","/s","/c"]).raw_arg(cmd_line(runner,id)?);spawn(c)}}
impl TerminalAdapter for PowerShellAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {let mut c=Command::new(&t.executable_path);c.args(["-NoLogo","-NoProfile","-Command"]).arg(format!("Start-Process -FilePath {} -ArgumentList '--run-job','{id}' -NoNewWindow -Wait",ps_quote(&runner.to_string_lossy())));spawn(c)}}
impl TerminalAdapter for WindowsTerminalAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {let system=std::env::var("SystemRoot").unwrap_or_else(|_|"C:\\Windows".into());let mut c=Command::new(&t.executable_path);c.args(["-w","new","new-tab"]).arg(Path::new(&system).join("System32/cmd.exe")).args(["/d","/s","/c"]).raw_arg(cmd_line(runner,id)?);spawn(c)}}
impl TerminalAdapter for GitBashAdapter {fn launch(&self,t:&TerminalInfo,runner:&Path,id:&str)->Result<()> {let mut c=Command::new(&t.executable_path);let r=runner.to_string_lossy().replace('\\',"/");c.args(["--no-cd","-c"]).arg(format!("{} --run-job {id}",sh_quote(&r)));spawn(c)}}
