use super::{err, Result};
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub fn open(path: &Path, folder: bool) -> Result<()> {
    let target = if folder {
        path.parent().ok_or("저장 폴더를 찾을 수 없습니다.")?
    } else {
        path
    };
    crate::security::require(
        target.exists(),
        "저장된 출력 파일이 없습니다. 실행 전이거나 이전 버전의 기록일 수 있습니다.",
    )?;
    let mut command = viewer(target, folder);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn().map_err(err)?;
    Ok(())
}
#[cfg(windows)]
fn viewer(target: &Path, folder: bool) -> Command {
    use std::os::windows::process::CommandExt;
    let system = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let executable = if folder {
        "explorer.exe"
    } else {
        "System32/notepad.exe"
    };
    let mut command = Command::new(Path::new(&system).join(executable));
    command.arg(target).creation_flags(0x08000000);
    command
}
#[cfg(target_os = "macos")]
fn viewer(target: &Path, folder: bool) -> Command {
    let mut command = Command::new("/usr/bin/open");
    if !folder {
        command.arg("-t");
    }
    command.arg(target);
    command
}
#[cfg(not(any(windows, target_os = "macos")))]
fn viewer(target: &Path, _: bool) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(target);
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_output_returns_error_without_launching_viewer() {
        let path = std::env::temp_dir()
            .join("PromptWakeup-missing-output-test")
            .join("missing.output");
        assert!(open(&path, false).is_err());
    }
    #[test]
    fn output_path_is_passed_as_one_argument() {
        let path = Path::new("folder with spaces/record.output");
        let command = viewer(path, false);
        assert_eq!(command.get_args().last().unwrap(), path.as_os_str());
    }
    #[cfg(windows)]
    #[test]
    fn output_uses_text_editor_and_folder_uses_explorer() {
        let path = Path::new("C:/example/record.output");
        assert!(Path::new(viewer(path, false).get_program()).ends_with("notepad.exe"));
        assert!(Path::new(viewer(path, true).get_program()).ends_with("explorer.exe"));
    }
}
