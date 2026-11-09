use super::*;
pub struct CodexAdapter;
pub struct ClaudeAdapter;
pub struct GeminiAdapter;
macro_rules! common {
    ($kind:expr) => {
        fn detect(&self) -> Option<String> {
            find($kind.name())
        }
        fn validate(&self, path: &str) -> Result<()> {
            validate_identity($kind, path)
        }
        fn get_version(&self, path: &str) -> Result<String> {
            version(path)
        }
    };
}
impl AiCliAdapter for CodexAdapter {
    common!(CliKind::Codex);
    fn build_launch_plan(&self, input: &ScheduleInput) -> LaunchPlan {
        plan(input, vec!["exec".into(), "-".into()])
    }
}
impl AiCliAdapter for ClaudeAdapter {
    common!(CliKind::Claude);
    fn build_launch_plan(&self, input: &ScheduleInput) -> LaunchPlan {
        plan(input, vec!["-p".into()])
    }
}
impl AiCliAdapter for GeminiAdapter {
    common!(CliKind::Gemini);
    fn build_launch_plan(&self, input: &ScheduleInput) -> LaunchPlan {
        plan(
            input,
            vec!["-p".into(), "Execute the task supplied on stdin".into()],
        )
    }
}
