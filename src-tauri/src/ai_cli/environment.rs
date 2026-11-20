use crate::storage::{err, Result};
use std::{
    ffi::OsString,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::OnceLock,
    time::{Duration, Instant},
};

// macOS: Recover the login PATH when launched from Finder.
pub(super) fn path() -> &'static OsString {
    static PATH: OnceLock<OsString> = OnceLock::new();
    PATH.get_or_init(|| {
        let inherited = std::env::var_os("PATH").unwrap_or_default();
        let login = login_path().unwrap_or_else(|| inherited.clone());
        merge_paths(&login, &inherited, super::search_directories()).unwrap_or(inherited)
    })
}

// CLI: Also search beside a manually selected executable for its runtime.
pub(super) fn configure(command: &mut Command, executable: &Path) -> Result<()> {
    let parent = executable.parent().into_iter().map(Path::to_path_buf);
    let search = std::env::join_paths(parent.chain(std::env::split_paths(path()))).map_err(err)?;
    command.env("PATH", search);
    Ok(())
}

// PATH: Preserve priority, excluding duplicates and relative directories.
fn merge_paths(
    login: &std::ffi::OsStr,
    inherited: &std::ffi::OsStr,
    fallback: Vec<PathBuf>,
) -> Option<OsString> {
    let mut paths = Vec::new();
    for entry in std::env::split_paths(login)
        .chain(std::env::split_paths(inherited))
        .chain(fallback)
    {
        if entry.is_absolute() && !paths.contains(&entry) {
            paths.push(entry);
        }
    }
    std::env::join_paths(paths).ok()
}

// macOS: Query PATH with a fixed command and no user input.
fn login_path() -> Option<OsString> {
    let shell = std::env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|shell| shell.is_absolute() && shell.is_file())
        .unwrap_or_else(|| PathBuf::from("/bin/zsh"));
    let mut output = tempfile::tempfile().ok()?;
    let mut child = Command::new(shell)
        .args(["-ilc", "printf '\\0%s\\0' \"$PATH\""])
        .current_dir(dirs::home_dir()?)
        .stdin(Stdio::null())
        .stdout(output.try_clone().ok()?)
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let success = wait_for_shell(&mut child);
    if !success {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    use std::io::{Seek, SeekFrom};
    output.seek(SeekFrom::Start(0)).ok()?;
    let mut bytes = Vec::new();
    output.take(64 * 1024).read_to_end(&mut bytes).ok()?;
    parse_path(&bytes)
}

// macOS: Bound the wait for unresponsive shell startup scripts.
fn wait_for_shell(child: &mut Child) -> bool {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(5) {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Err(_) => return false,
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
        }
    }
    false
}

// PATH: Preserve raw path bytes separately from shell startup messages.
fn parse_path(bytes: &[u8]) -> Option<OsString> {
    use std::os::unix::ffi::OsStringExt;
    let value = bytes.split(|byte| *byte == 0).nth(1)?;
    (!value.is_empty()).then(|| OsString::from_vec(value.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_path_preserves_spaces_and_ignores_shell_messages() {
        assert_eq!(
            parse_path(b"welcome\n\0/Users/test user/.nvm/bin:/usr/bin\0goodbye\n"),
            Some(OsString::from("/Users/test user/.nvm/bin:/usr/bin"))
        );
        assert_eq!(parse_path(b"shell failed"), None);
        assert_eq!(parse_path(b"\0\0"), None);
    }

    #[test]
    fn path_keeps_login_priority_and_excludes_working_directory() {
        let merged = merge_paths(
            "/custom/node/bin:/usr/bin".as_ref(),
            "/usr/bin::.:relative:/bin".as_ref(),
            vec![PathBuf::from("/opt/homebrew/bin")],
        )
        .unwrap();
        assert_eq!(merged, "/custom/node/bin:/usr/bin:/bin:/opt/homebrew/bin");
    }

    #[test]
    fn manual_cli_directory_is_available_to_env_shebang() {
        let directory = tempfile::tempdir().unwrap();
        let cli = directory.path().join("codex");
        let runtime = directory.path().join("prompt-wakeup-test-runtime");
        std::fs::write(&cli, "#!/usr/bin/env prompt-wakeup-test-runtime\n").unwrap();
        std::fs::write(&runtime, "#!/bin/sh\nprintf 'runtime found'\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        for file in [&cli, &runtime] {
            std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let mut command = super::super::command(cli.to_str().unwrap(), &[]).unwrap();
        let output = command.output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"runtime found");
    }
}
