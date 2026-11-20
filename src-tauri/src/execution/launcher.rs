use crate::storage::{err, Result};

// 실행: 작업 프로세스 시작.
pub fn launch(id: &str) -> Result<()> {
    uuid::Uuid::parse_str(id).map_err(|_| "실행 ID 오류")?;
    let runner = std::env::current_exe().map_err(err)?;
    let mut command = std::process::Command::new(runner);
    command
        .args(["--run-job", id])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(err)?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
