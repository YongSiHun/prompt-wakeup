import { embeddedTerminal } from "../../src/lib/embedded_terminal";
import type {
  CliInfo,
  ExecutionRecord,
  Schedule,
  Settings,
  Snapshot,
} from "../../src/types";
export function makeSettings(overrides: Partial<Settings> = {}): Settings {
  return {
    default_cli: "codex",
    default_terminal: "embedded",
    default_directory: "C:/작업 폴더",
    theme: "light",
    keep_open: false,
    notifications: false,
    retention_days: 90,
    custom_cli_paths: {},
    custom_terminals: [],
    ...overrides,
  };
}
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
export function makeSchedule(overrides: Partial<Schedule> = {}): Schedule {
  return {
    id: "job-1",
    name: "예약 테스트",
    cli: "codex",
    cli_path: "C:/tools/codex.exe",
    terminal: { ...embeddedTerminal },
    working_directory: "C:/작업",
    prompt: "작업",
    scheduled_at: "2037-01-01T00:00:00Z",
    timezone: "Asia/Seoul",
    local_time: "2037-01-01T09:00:00",
    keep_open: false,
    status: "pending",
    ...overrides,
  };
}
export function makeExecution(
  overrides: Partial<ExecutionRecord> = {},
): ExecutionRecord {
  return {
    id: "run-1",
    schedule_id: "job-1",
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
