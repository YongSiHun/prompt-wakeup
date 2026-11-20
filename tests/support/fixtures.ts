import type {
  CliInfo,
  ExecutionRecord,
  Schedule,
  Settings,
  Snapshot,
} from "../../src/types";
// Test: make settings.
export function makeSettings(overrides: Partial<Settings> = {}): Settings {
  return {
    default_cli: "codex",
    default_directory: "C:/작업 폴더",
    theme: "light",
    retention_days: 90,
    custom_cli_paths: {},
    ...overrides,
  };
}
// Test: make cli.
export function makeCli(overrides: Partial<CliInfo> = {}): CliInfo {
  return {
    kind: "codex",
    executable_path: "C:/tools/codex.exe",
    available: true,
    version: "1.0",
    validation_error: null,
    ...overrides,
  };
}
// Test: make schedule.
export function makeSchedule(overrides: Partial<Schedule> = {}): Schedule {
  return {
    id: "job-1",
    name: "예약 테스트",
    cli: "codex",
    cli_path: "C:/tools/codex.exe",
    working_directory: "C:/작업",
    prompt: "작업",
    scheduled_at: "2037-01-01T00:00:00Z",
    timezone: "Asia/Seoul",
    local_time: "2037-01-01T09:00:00",
    status: "pending",
    ...overrides,
  };
}
// Test: make execution.
export function makeExecution(
  overrides: Partial<ExecutionRecord> = {},
): ExecutionRecord {
  return {
    id: "run-1",
    schedule_id: "job-1",
    prompt: "실행 시점의 프롬프트",
    name: "첫 작업",
    cli: "codex",
    started_at: "2026-01-01T00:00:00Z",
    ended_at: null,
    status: "launched",
    exit_code: null,
    error: null,
    ...overrides,
  };
}
// Test: make snapshot.
export function makeSnapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    active: true,
    warnings: [],
    settings: makeSettings(),
    schedules: [],
    executions: [],
    ...overrides,
  };
}
