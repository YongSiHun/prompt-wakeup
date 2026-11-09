export type CliKind = "codex" | "claude" | "gemini";
export type Status =
  | "pending"
  | "launching"
  | "launched"
  | "completed"
  | "failed"
  | "canceled"
  | "missed";
export interface TerminalInfo {
  id: string;
  name: string;
  kind: string;
  executable_path: string;
  detected_automatically: boolean;
  installed: boolean;
  supported: boolean;
  validation_error: string | null;
}
export interface CliInfo {
  kind: CliKind;
  executable_path: string;
  available: boolean;
  version: string | null;
  validation_error: string | null;
}
export interface ScheduleInput {
  name: string;
  cli: CliKind;
  cli_path: string;
  terminal: TerminalInfo;
  working_directory: string;
  prompt: string;
  scheduled_at: string;
  timezone: string;
  local_time: string;
  keep_open: boolean;
}
export interface Schedule extends ScheduleInput {
  id: string;
  status: Status;
}
export interface ExecutionRecord {
  id: string;
  schedule_id: string;
  name: string;
  cli: CliKind;
  started_at: string;
  ended_at: string | null;
  status:
    | Exclude<Status, "pending" | "canceled" | "missed">
    | "unknown"
    | "tracking_unavailable";
  exit_code: number | null;
  error: string | null;
}
export interface Settings {
  custom_cli_paths: Record<string, string>;
  custom_terminals: TerminalInfo[];
  default_cli: CliKind;
  default_terminal: string;
  default_directory: string;
  theme: "system" | "light" | "dark";
  keep_open: boolean;
  notifications: boolean;
  retention_days: number;
}
export interface Snapshot {
  schedules: Schedule[];
  executions: ExecutionRecord[];
  settings: Settings;
  active: boolean;
  warnings: string[];
}
