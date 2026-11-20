import type {
  Schedule,
  Settings,
  CliInfo,
  CliKind,
  ScheduleInput,
} from "../../types";
import { embeddedTerminal } from "../../lib/embedded_terminal";
import { localInput, utcFromLocal } from "../../lib/time";

export interface ScheduleDraft {
  name: string;
  cli: CliKind | "";
  cliPath: string;
  directory: string;
  prompt: string;
  time: string;
}
export function initialDraft(
  schedule: Schedule | undefined,
  settings: Settings,
): ScheduleDraft {
  if (schedule) return existingDraft(schedule);
  return defaultDraft(settings);
}
function existingDraft(schedule: Schedule): ScheduleDraft {
  return {
    name: schedule.name,
    cli: schedule.cli,
    cliPath: schedule.cli_path,
    directory: schedule.working_directory,
    prompt: schedule.prompt,
    time: localInput(new Date(schedule.scheduled_at)),
  };
}
function defaultDraft(settings: Settings): ScheduleDraft {
  return {
    name: "새 예약 작업",
    cli: settings.default_cli ?? "",
    cliPath: "",
    directory: settings.default_directory,
    prompt: "",
    time: localInput(new Date(Date.now() + 3600_000)),
  };
}
function selectedCliPath(draft: ScheduleDraft, clis: CliInfo[]): string {
  if (draft.cliPath) return draft.cliPath;
  const cli = clis.find((cli) => cli.kind === draft.cli);
  if (!cli?.available)
    throw new Error("CLI를 설치하거나 실행 파일을 선택하세요.");
  return cli.executable_path;
}
export function buildInput(
  draft: ScheduleDraft,
  clis: CliInfo[],
): ScheduleInput {
  if (!draft.cli) throw new Error("AI CLI를 선택하세요.");
  const time = normalizeTime(draft.time);
  return {
    name: draft.name,
    cli: draft.cli,
    cli_path: selectedCliPath(draft, clis),
    terminal: embeddedTerminal,
    working_directory: draft.directory,
    prompt: draft.prompt,
    scheduled_at: utcFromLocal(time),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    local_time: time,
    keep_open: false,
  };
}
export function normalizeTime(value: string) {
  const [date, time = ""] = value.split("T");
  const parts = time.split(":");
  if (parts.length === 2) parts.push("00");
  return date + "T" + parts.map((part) => part.padStart(2, "0")).join(":");
}

export function nextScheduleNumber(
  schedules: Schedule[],
  sequence = 0,
): number {
  return (
    schedules.reduce(
      (max, schedule) => {
        const match = /^새 예약 작업 (\d+)$/.exec(schedule.name.trim());
        return Math.max(max, match ? Number(match[1]) : 0);
      },
      Math.max(sequence, schedules.length),
    ) + 1
  );
}
