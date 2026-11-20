use super::{err, Result};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    process::{Command, Stdio},
};

const PREVIEW_BYTES: u64 = 256 * 1024;

// I/O: 제한된 최근 출력 조회.
pub fn read_preview(path: &Path) -> Result<String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(error) => return Err(err(error)),
    };
    let size = file.metadata().map_err(err)?.len();
    let start = size.saturating_sub(PREVIEW_BYTES);
    file.seek(SeekFrom::Start(start)).map_err(err)?;
    let mut bytes = Vec::with_capacity((size - start) as usize);
    file.take(PREVIEW_BYTES)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    Ok(preview_text(&bytes, start > 0))
}

// 변환: UTF-8 경계·생략 안내.
fn preview_text(bytes: &[u8], truncated: bool) -> String {
    if !truncated {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let boundary = bytes
        .iter()
        .position(|byte| byte & 0xc0 != 0x80)
        .unwrap_or(bytes.len());
    format!(
        "[최근 출력만 표시합니다. 전체 내용은 파일 열기로 확인하세요.]\n{}",
        String::from_utf8_lossy(&bytes[boundary..])
    )
}

// I/O: 저장소·파일 열기.
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
// OS: 출력 뷰어 명령.
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
// OS: 출력 뷰어 명령.
fn viewer(target: &Path, folder: bool) -> Command {
    let mut command = Command::new("/usr/bin/open");
    if !folder {
        command.arg("-t");
    }
    command.arg(target);
    command
}
#[cfg(not(any(windows, target_os = "macos")))]
// OS: 출력 뷰어 명령.
fn viewer(target: &Path, _: bool) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(target);
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // Test: preview is bounded and preserves original unicode file.
    fn preview_is_bounded_and_preserves_original_unicode_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("large.output");
        let text = "한글".repeat(100_000);
        std::fs::write(&path, &text).unwrap();
        let preview = read_preview(&path).unwrap();
        assert!(preview.starts_with("[최근 출력만 표시합니다."));
        assert!(preview.len() < PREVIEW_BYTES as usize + 200);
        assert!(preview.ends_with("한글"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
    #[test]
    // Test: UTF-8 경계를 넘는 출력 미리보기.
    fn truncated_preview_starts_at_unicode_boundary() {
        assert_eq!(
            preview_text(&[0x95, 0x9c, 0xea, 0xb8, 0x80], true)
                .chars()
                .last(),
            Some('글')
        );
        assert!(!preview_text(&[0x95, 0x9c, 0xea, 0xb8, 0x80], true)
            .contains(char::REPLACEMENT_CHARACTER));
    }
    #[test]
    // Test: small preview preserves all output and missing file is empty.
    fn small_preview_preserves_all_output_and_missing_file_is_empty() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("small.output");
        assert_eq!(read_preview(&path).unwrap(), "");
        std::fs::write(&path, "한글\nstdout\nstderr").unwrap();
        assert_eq!(read_preview(&path).unwrap(), "한글\nstdout\nstderr");
    }
    #[test]
    // Test: missing output returns error without launching viewer.
    fn missing_output_returns_error_without_launching_viewer() {
        let path = std::env::temp_dir()
            .join("PromptWakeup-missing-output-test")
            .join("missing.output");
        assert!(open(&path, false).is_err());
    }
    #[test]
    // Test: output path is passed as one argument.
    fn output_path_is_passed_as_one_argument() {
        let path = Path::new("folder with spaces/record.output");
        let command = viewer(path, false);
        assert_eq!(command.get_args().last().unwrap(), path.as_os_str());
    }
    #[cfg(windows)]
    #[test]
    // Test: output uses text editor and folder uses explorer.
    fn output_uses_text_editor_and_folder_uses_explorer() {
        let path = Path::new("C:/example/record.output");
        assert!(Path::new(viewer(path, false).get_program()).ends_with("notepad.exe"));
        assert!(Path::new(viewer(path, true).get_program()).ends_with("explorer.exe"));
    }
}
