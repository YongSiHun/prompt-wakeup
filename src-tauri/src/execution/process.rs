use crate::{
    models::ScheduleInput,
    security::SecurityValidator,
    storage::{err, Result},
};
use std::{
    fs,
    io::Write,
    process::{ExitStatus, Stdio},
};

pub(super) fn execute(input: &ScheduleInput, output: &fs::File) -> Result<Option<i32>> {
    SecurityValidator::input(input, false)?;
    let plan = crate::ai_cli::adapter(input.cli).build_launch_plan(input);
    let mut command = crate::ai_cli::command(&plan.executable, &plan.arguments)?;
    command
        .current_dir(&plan.working_directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(output.try_clone().map_err(err)?))
        .stderr(Stdio::from(output.try_clone().map_err(err)?));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(err)?;
    let mut stdin = child.stdin.take().ok_or("CLI stdin 없음")?;
    std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(input.prompt.as_bytes()));
        let status = child.wait().map_err(err)?;
        let delivered = writer.join().map_err(|_| "프롬프트 전달 스레드 오류")?;
        validate_delivery(status, delivered)?;
        Ok(status.code())
    })
}

fn validate_delivery(status: ExitStatus, delivered: std::io::Result<()>) -> Result<()> {
    if status.success() {
        delivered.map_err(err)?;
    }
    Ok(())
}
