use super::*;
use crate::test_support::input;

#[test]
// Test: utc local roundtrip and dst rejection.
fn utc_local_roundtrip_and_dst_rejection() {
    let directory = tempfile::tempdir().unwrap();
    let mut input = input(directory.path());
    assert!(SecurityValidator::input(&input, true).is_ok());
    input.timezone = "America/New_York".into();
    input.local_time = "2027-11-07T01:30:00".into();
    input.scheduled_at = "2027-11-07T05:30:00Z".parse().unwrap();
    assert!(SecurityValidator::input(&input, false)
        .unwrap_err()
        .contains("DST"));
    input.local_time = "2027-03-14T02:30:00".into();
    assert!(SecurityValidator::input(&input, false).is_err());
}

#[test]
// Test: invalid path and empty prompt rejected.
fn invalid_path_and_empty_prompt_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let mut input = input(directory.path());
    input.prompt = " ".into();
    assert!(SecurityValidator::input(&input, true).is_err());
    input.prompt = "ok".into();
    input.cli_path = "relative/codex".into();
    assert!(SecurityValidator::input(&input, true).is_err());
}
