import { useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useExecutablePicker } from "../../hooks/use_executable_picker";
import { useAppStore } from "../../stores/app_store";
import type {
  Schedule,
  CliKind,
  Settings,
  TerminalInfo,
  CliInfo,
  ScheduleInput,
} from "../../types";
import { defaultTerminalId } from "../../lib/executables";
import { localInput, utcFromLocal } from "../../lib/time";
interface Draft {
  name: string;
  cli: CliKind;
  cliPath: string;
  terminal: string;
  directory: string;
  prompt: string;
  time: string;
  keepOpen: boolean;
}
function initialDraft(
  schedule: Schedule | undefined,
  settings: Settings,
  terminals: TerminalInfo[],
): Draft {
  if (schedule) return existingDraft(schedule);
  return defaultDraft(settings, terminals);
}
function existingDraft(schedule: Schedule): Draft {
  return {
    name: schedule.name,
    cli: schedule.cli,
    cliPath: schedule.cli_path,
    terminal: schedule.terminal.id,
    directory: schedule.working_directory,
    prompt: schedule.prompt,
    time: localInput(new Date(schedule.scheduled_at)),
    keepOpen: schedule.keep_open,
  };
}
function defaultDraft(settings: Settings, terminals: TerminalInfo[]): Draft {
  return {
    name: "",
    cli: settings.default_cli,
    cliPath: "",
    terminal: defaultTerminalId(terminals, settings.default_terminal),
    directory: settings.default_directory,
    prompt: "",
    time: localInput(new Date(Date.now() + 3600_000)),
    keepOpen: settings.keep_open,
  };
}
function formTerminals(
  schedule: Schedule | undefined,
  terminals: TerminalInfo[],
) {
  if (!schedule) return terminals;
  if (terminals.some((terminal) => terminal.id === schedule.terminal.id))
    return terminals;
  return [...terminals, schedule.terminal];
}
function selectedCliPath(draft: Draft, clis: CliInfo[]): string {
  if (draft.cliPath) return draft.cliPath;
  const cli = clis.find((cli) => cli.kind === draft.cli);
  if (!cli?.available)
    throw new Error("CLI를 설치하거나 실행 파일을 선택하세요.");
  return cli.executable_path;
}
function buildInput(
  draft: Draft,
  terminals: TerminalInfo[],
  clis: CliInfo[],
): ScheduleInput {
  const terminal = terminals.find((terminal) => terminal.id === draft.terminal);
  if (!terminal?.supported) throw new Error("지원되는 터미널을 선택하세요.");
  return {
    name: draft.name,
    cli: draft.cli,
    cli_path: selectedCliPath(draft, clis),
    terminal,
    working_directory: draft.directory,
    prompt: draft.prompt,
    scheduled_at: utcFromLocal(draft.time),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    local_time: draft.time,
    keep_open: draft.keepOpen,
  };
}
function cliPath(clis: CliInfo[], kind: CliKind) {
  return (
    clis.find((cli) => cli.kind === kind)?.executable_path ||
    "AI CLI 실행 파일을 선택하세요."
  );
}
function terminalPath(terminals: TerminalInfo[], id: string) {
  return (
    terminals.find((terminal) => terminal.id === id)?.executable_path ||
    "터미널 실행 파일을 선택하세요."
  );
}
function normalizeTime(value: string) {
  return value.length === 16 ? value + ":00" : value;
}
async function chooseFolder(onSelect: (path: string) => void) {
  try {
    const path = await open({ directory: true, multiple: false });
    if (typeof path === "string") onSelect(path);
  } catch (error) {
    useAppStore.getState().setError(String(error));
  }
}
export function ScheduleForm({
  schedule,
  onClose,
}: {
  schedule?: Schedule;
  onClose: () => void;
}) {
  const settings = useAppStore((state) => state.data!.settings);
  const clis = useAppStore((state) => state.clis);
  const terminals = useAppStore((state) => state.terminals);
  const { picking, chooseTerminal, chooseCli } = useExecutablePicker();
  const busy = useAppStore((state) => state.busy) || picking;
  const [draft, setDraft] = useState(() =>
    initialDraft(schedule, settings, terminals),
  );
  const update = <K extends keyof Draft>(key: K, value: Draft[K]) =>
    setDraft((current) => ({ ...current, [key]: value }));
  const available = formTerminals(schedule, terminals);
  const selectedTerminal =
    draft.terminal || defaultTerminalId(available, settings.default_terminal);
  async function submit(event: FormEvent) {
    event.preventDefault();
    const store = useAppStore.getState();
    try {
      const input = buildInput(
        { ...draft, terminal: selectedTerminal },
        available,
        clis,
      );
      const saved = await store.perform("save_schedule", {
        id: schedule?.id ?? null,
        input,
      });
      if (saved) onClose();
    } catch (error) {
      store.setError(String(error));
    }
  }
  return (
    <div className="overlay">
      <section
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="form-title"
      >
        <header>
          <h2 id="form-title">{schedule ? "예약 수정" : "새 프롬프트 예약"}</h2>
          <button onClick={onClose} aria-label="닫기">
            ×
          </button>
        </header>
        <form onSubmit={submit}>
          <label>
            작업 이름
            <input
              required
              maxLength={100}
              value={draft.name}
              onChange={(event) => update("name", event.target.value)}
              autoFocus
            />
          </label>
          <div className="grid">
            <label>
              AI CLI
              <select
                disabled={busy}
                value={draft.cli}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    cli: event.target.value as CliKind,
                    cliPath: "",
                  }))
                }
              >
                {(["codex", "claude", "gemini"] as const).map((kind) => (
                  <option key={kind} value={kind}>
                    {kind}
                    {clis.find((cli) => cli.kind === kind)?.available
                      ? ""
                      : " (사용 불가)"}
                  </option>
                ))}
              </select>
            </label>
            <label>
              외부 터미널
              <select
                required
                disabled={busy}
                value={selectedTerminal}
                onChange={(event) => update("terminal", event.target.value)}
              >
                <option value="">터미널 선택</option>
                {available.map((terminal) => (
                  <option
                    key={terminal.id}
                    value={terminal.id}
                    disabled={!terminal.supported}
                  >
                    {terminal.name}
                    {terminal.detected_automatically
                      ? ""
                      : " · " + terminal.executable_path}
                    {terminal.supported ? "" : " (지원 불가)"}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <div className="grid">
            <div className="executable-path">
              <button
                type="button"
                disabled={busy}
                onClick={async () => {
                  const cli = await chooseCli(draft.cli);
                  if (cli) update("cliPath", cli.executable_path);
                }}
              >
                AI CLI 실행 파일 선택
              </button>
              <p className="hint executable-path">
                {draft.cliPath || cliPath(clis, draft.cli)}
              </p>
            </div>
            <div className="executable-path">
              <button
                type="button"
                disabled={busy}
                onClick={async () => {
                  const terminal = await chooseTerminal();
                  if (terminal) update("terminal", terminal.id);
                }}
              >
                터미널 실행 파일 선택
              </button>
              <p className="hint executable-path">
                {terminalPath(available, selectedTerminal)}
              </p>
            </div>
          </div>
          <label>
            작업 디렉터리
            <div className="row">
              <input
                required
                value={draft.directory}
                onChange={(event) => update("directory", event.target.value)}
              />
              <button
                type="button"
                onClick={() =>
                  void chooseFolder((path) => update("directory", path))
                }
              >
                폴더 선택
              </button>
            </div>
          </label>
          <label>
            프롬프트
            <textarea
              required
              rows={7}
              value={draft.prompt}
              onChange={(event) => update("prompt", event.target.value)}
              placeholder="예약 시간에 CLI가 처리할 작업을 입력하세요."
            />
          </label>
          <label>
            실행 날짜 및 시각 ·{" "}
            {Intl.DateTimeFormat().resolvedOptions().timeZone}
            <input
              required
              type="datetime-local"
              step="1"
              value={draft.time}
              onChange={(event) =>
                update("time", normalizeTime(event.target.value))
              }
            />
          </label>
          <label className="check">
            <input
              type="checkbox"
              checked={draft.keepOpen}
              onChange={(event) => update("keepOpen", event.target.checked)}
            />
            종료 후 터미널 유지
          </label>
          <p className="hint">
            프롬프트는 이 PC의 사용자 데이터 폴더에 저장됩니다. 앱 실행 중에만
            예약이 유효하며, 절전 중 지난 작업은 자동 실행하지 않습니다.
          </p>
          <footer>
            <button type="button" onClick={onClose}>
              취소
            </button>
            <button className="primary" disabled={busy}>
              예약 저장
            </button>
          </footer>
        </form>
      </section>
    </div>
  );
}
