export const cliKinds = ["codex", "claude", "gemini"] as const;
export type CliKind = (typeof cliKinds)[number];
export type Status =
  | "pending"
  | "launching"
  | "launched"
  | "completed"
  | "failed"
  | "canceled"
  | "missed";
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
  working_directory: string;
  prompt: string;
  scheduled_at: string;
  timezone: string;
  local_time: string;
}
export interface Schedule extends ScheduleInput {
  id: string;
  status: Status;
}
export interface ExecutionRecord {
  prompt: string | null;
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
  default_cli: CliKind | null;
  default_directory: string;
  theme: "system" | "light" | "dark";
  retention_days: number;
}
export interface Snapshot {
  schedules: Schedule[];
  executions: ExecutionRecord[];
  settings: Settings;
  active: boolean;
  warnings: string[];
}
