use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleStatus {
    Pending,
    Launching,
    Launched,
    Completed,
    Failed,
    Canceled,
    Missed,
}
impl ScheduleStatus {
    // 상태: 실행 중 여부.
    pub fn is_running(self) -> bool {
        matches!(self, Self::Launching | Self::Launched)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Launching,
    Launched,
    Completed,
    Failed,
    Unknown,
    TrackingUnavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CliKind {
    Codex,
    Claude,
    Gemini,
}
impl CliKind {
    pub const ALL: [Self; 3] = [Self::Codex, Self::Claude, Self::Gemini];
    // 변환: CLI 이름.
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleInput {
    pub name: String,
    pub cli: CliKind,
    pub cli_path: String,
    pub working_directory: String,
    pub prompt: String,
    pub scheduled_at: DateTime<Utc>,
    pub timezone: String,
    pub local_time: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Schedule {
    pub id: String,
    #[serde(flatten)]
    pub input: ScheduleInput,
    pub status: ScheduleStatus,
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
    #[serde(default)]
    pub prompt: Option<String>,
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
    // 상태: 초기값 구성.
    pub fn new(schedule: &Schedule) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            schedule_id: schedule.id.clone(),
            name: schedule.input.name.clone(),
            cli: schedule.input.cli,
            prompt: Some(schedule.input.prompt.clone()),
            started_at: Utc::now(),
            ended_at: None,
            status: ExecutionStatus::Launching,
            exit_code: None,
            error: None,
        }
    }
    // 상태: 추적 중 여부.
    pub fn active(&self) -> bool {
        self.ended_at.is_none()
            && matches!(
                self.status,
                ExecutionStatus::Launching | ExecutionStatus::Launched | ExecutionStatus::Unknown
            )
    }
    // 변환: 실행→예약 상태.
    pub fn schedule_status(&self) -> ScheduleStatus {
        match self.status {
            ExecutionStatus::Completed => ScheduleStatus::Completed,
            ExecutionStatus::Launching => ScheduleStatus::Launching,
            ExecutionStatus::Launched => ScheduleStatus::Launched,
            _ => ScheduleStatus::Failed,
        }
    }
    // 상태: 종료 추적 실패 기록.
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
    pub default_cli: Option<CliKind>,
    pub default_directory: String,
    pub theme: String,
    pub retention_days: u32,
}
impl Default for Settings {
    // 설정: 기본값.
    fn default() -> Self {
        Self {
            custom_cli_paths: BTreeMap::new(),
            default_cli: None,
            default_directory: dirs::home_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            theme: "system".into(),
            retention_days: 90,
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
