use crate::models::*;
use chrono::{DateTime, Utc};

// Test: input.
pub(crate) fn input(directory: &std::path::Path) -> ScheduleInput {
    let cli = directory.join(if cfg!(windows) { "codex.cmd" } else { "codex" });
    std::fs::write(&cli, version_fixture()).unwrap();
    make_executable(&cli);
    let time =
        DateTime::from_timestamp((Utc::now() + chrono::Duration::hours(1)).timestamp(), 0).unwrap();
    ScheduleInput {
        name: "한글 예약".into(),
        cli: CliKind::Codex,
        cli_path: cli.to_string_lossy().into(),
        working_directory: directory.to_string_lossy().into(),
        prompt: "한글 ' \" & $(touch bad) \n다음 줄 %PATH%".into(),
        scheduled_at: time,
        timezone: "Asia/Seoul".into(),
        local_time: local_time(time),
    }
}
// Test: local time.
pub(crate) fn local_time(time: DateTime<Utc>) -> String {
    time.with_timezone(&chrono_tz::Asia::Seoul)
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string()
}
// Test: version fixture.
fn version_fixture() -> &'static str {
    if cfg!(windows) {
        "@echo off\r\necho fake-cli 1.0\r\nexit /b 0\r\n"
    } else {
        "#!/bin/sh\necho fake-cli 1.0\n"
    }
}
#[cfg(unix)]
// Test: make executable.
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}
#[cfg(not(unix))]
// Test: make executable.
fn make_executable(_: &std::path::Path) {}
