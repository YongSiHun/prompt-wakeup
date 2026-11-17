use super::*;
use crate::test_support::input;

#[test]
fn adapters_keep_prompts_out_of_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let input = input(directory.path());
    let plans = [
        crate::ai_cli::adapters::CodexAdapter.build_launch_plan(&input),
        crate::ai_cli::adapters::ClaudeAdapter.build_launch_plan(&input),
        crate::ai_cli::adapters::GeminiAdapter.build_launch_plan(&input),
    ];
    assert_adapter_arguments(&plans);
    for plan in plans {
        assert_prompt_delivery(&plan, &input.prompt);
    }
}

#[test]
fn version_detects_stub_cli() {
    let directory = tempfile::tempdir().unwrap();
    assert!(crate::ai_cli::version(&input(directory.path()).cli_path)
        .unwrap()
        .contains("fake-cli"));
}

#[test]
fn verbose_version_output_does_not_fill_the_pipe() {
    let directory = tempfile::tempdir().unwrap();
    let input = input(directory.path());
    let script = if cfg!(windows) {
        "@echo off\r\nfor /L %%i in (1,1,20000) do @echo fake-cli version output\r\nexit /b 0\r\n"
    } else {
        "#!/bin/sh\ni=0\nwhile [ $i -lt 20000 ]; do echo fake-cli version output; i=$((i+1)); done\n"
    };
    std::fs::write(&input.cli_path, script).unwrap();
    let version = crate::ai_cli::version(&input.cli_path).unwrap();
    assert!(version.starts_with("fake-cli"));
    assert!(version.len() <= 4096);
}

fn assert_adapter_arguments(plans: &[LaunchPlan; 3]) {
    assert_eq!(plans[0].arguments, vec!["exec", "-"]);
    assert_eq!(plans[1].arguments, vec!["-p"]);
    assert_eq!(
        plans[2].arguments,
        vec!["-p", "Execute the task supplied on stdin"]
    );
}

fn assert_prompt_delivery(plan: &LaunchPlan, prompt: &str) {
    assert!(!plan.arguments.iter().any(|argument| argument == prompt));
}
