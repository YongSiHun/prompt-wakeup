use super::command;
use crate::{
    security::require,
    storage::{err, Result},
};
use std::{
    io::Read,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

const VERSION_TIMEOUT: Duration = Duration::from_secs(5);

pub fn version(path: &str) -> Result<String> {
    let started = Instant::now();
    let mut command = command(path, &["--version".into()])?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    hide_probe(&mut command);
    let mut child = command.spawn().map_err(err)?;
    let stdout = child.stdout.take().ok_or("CLI 출력 채널이 없습니다.")?;
    // Drain stdout while waiting: a full pipe otherwise blocks the CLI from exiting.
    let (sender, output) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read_version_output(stdout));
    });
    let status = wait_for_version(&mut child)?;
    require(
        status.success(),
        "CLI 버전 검사 실패. 터미널에서 설치 및 로그인을 확인하세요.",
    )?;
    output
        .recv_timeout(VERSION_TIMEOUT.saturating_sub(started.elapsed()))
        .map_err(|_| "CLI 출력 확인 시간 초과 또는 출력 처리 오류")?
}
fn read_version_output(mut stdout: impl Read) -> Result<String> {
    let mut preview = Vec::new();
    (&mut stdout)
        .take(4096)
        .read_to_end(&mut preview)
        .map_err(err)?;
    std::io::copy(&mut stdout, &mut std::io::sink()).map_err(err)?;
    Ok(String::from_utf8_lossy(&preview).trim().into())
}
fn wait_for_version(child: &mut Child) -> Result<ExitStatus> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(err)? {
            return Ok(status);
        }
        if start.elapsed() >= VERSION_TIMEOUT {
            return version_timeout(child);
        }
        std::thread::sleep(Duration::from_millis(40));
    }
}
fn version_timeout(child: &mut Child) -> Result<ExitStatus> {
    let _ = child.kill();
    let _ = child.wait();
    Err("CLI 버전 확인 시간 초과".into())
}
#[cfg(windows)]
fn hide_probe(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
}
#[cfg(not(windows))]
fn hide_probe(_: &mut Command) {}
