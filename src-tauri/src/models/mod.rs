use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleStatus { Pending, Launching, Launched, Completed, Failed, Canceled, Missed }
impl ScheduleStatus {
    pub fn is_running(self) -> bool { matches!(self, Self::Launching | Self::Launched) }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus { Launching, Launched, Completed, Failed, Unknown, TrackingUnavailable }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CliKind { Codex, Claude, Gemini }
impl CliKind {
    pub const ALL: [Self; 3] = [Self::Codex, Self::Claude, Self::Gemini];
    pub fn name(self) -> &'static str {
        match self { Self::Codex => "codex", Self::Claude => "claude", Self::Gemini => "gemini" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleInput {
    pub name: String,
    pub cli: CliKind,
    pub cli_path: String,
    pub terminal: TerminalInfo,
    pub working_directory: String,
    pub prompt: String,
    pub scheduled_at: DateTime<Utc>,
    pub timezone: String,
    pub local_time: String,
    pub keep_open: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Schedule {
    pub id: String,
    #[serde(flatten)]
    pub input: ScheduleInput,
    pub status: ScheduleStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub executable_path: String,
    pub detected_automatically: bool,
    pub installed: bool,
    pub supported: bool,
    pub validation_error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliInfo {
    pub kind: CliKind,
    pub executable_path: String,
    pub available: bool,
    pub version: Option<String>,
    pub validation_error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub id: String,
    pub schedule_id: String,
    pub name: String,
    pub cli: CliKind,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}
impl ExecutionRecord {
    pub fn new(schedule: &Schedule) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(), schedule_id: schedule.id.clone(),
            name: schedule.input.name.clone(), cli: schedule.input.cli, started_at: Utc::now(),
            ended_at: None, status: ExecutionStatus::Launching, exit_code: None, error: None,
        }
    }
    pub fn active(&self) -> bool {
        self.ended_at.is_none() && matches!(self.status, ExecutionStatus::Launching | ExecutionStatus::Launched | ExecutionStatus::Unknown)
    }
    pub fn schedule_status(&self) -> ScheduleStatus {
        match self.status {
            ExecutionStatus::Completed => ScheduleStatus::Completed,
            ExecutionStatus::Launching => ScheduleStatus::Launching,
            ExecutionStatus::Launched => ScheduleStatus::Launched,
            _ => ScheduleStatus::Failed,
        }
    }
    pub fn mark_unknown(&mut self, now: DateTime<Utc>, error: String) {
        self.status = ExecutionStatus::Unknown;
        self.ended_at = Some(now);
        self.error = Some(error);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub custom_cli_paths: BTreeMap<String, String>,
    pub custom_terminals: Vec<TerminalInfo>,
    pub default_cli: CliKind,
    pub default_terminal: String,
    pub default_directory: String,
    pub theme: String,
    pub keep_open: bool,
    pub notifications: bool,
    pub retention_days: u32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            custom_cli_paths: BTreeMap::new(), custom_terminals: Vec::new(),
            default_cli: CliKind::Codex,
            default_terminal: if cfg!(windows) { "cmd" } else { "terminal" }.into(),
            default_directory: dirs::home_dir().unwrap_or_default().to_string_lossy().into(),
            theme: "system".into(), keep_open: true, notifications: true, retention_days: 90,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schedules: Vec<Schedule>,
    pub executions: Vec<ExecutionRecord>,
    pub settings: Settings,
    pub active: bool,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchPlan {
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: String,
    pub environment: BTreeMap<String, String>,
    pub terminal_adapter_id: String,
    pub prompt_delivery: String,
    pub status_tracking_strategy: String,
    pub close_behavior: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub execution_id: String,
    pub schedule: ScheduleInput,
    pub expires_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub execution_id: String,
    pub ended_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}
