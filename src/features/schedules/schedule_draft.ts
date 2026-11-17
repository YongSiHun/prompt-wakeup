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
  cli: CliKind;
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
    name: "",
    cli: settings.default_cli,
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
  return {
    name: draft.name,
    cli: draft.cli,
    cli_path: selectedCliPath(draft, clis),
    terminal: embeddedTerminal,
    working_directory: draft.directory,
    prompt: draft.prompt,
    scheduled_at: utcFromLocal(draft.time),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    local_time: draft.time,
    keep_open: false,
  };
}
export function cliPath(clis: CliInfo[], kind: CliKind) {
  return (
    clis.find((cli) => cli.kind === kind)?.executable_path ||
    "AI CLI 실행 파일을 선택하세요."
  );
}
export function normalizeTime(value: string) {
  return value.length === 16 ? value + ":00" : value;
}
