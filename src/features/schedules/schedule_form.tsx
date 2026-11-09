import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../../stores/app_store";
import type { Schedule, CliKind } from "../../types";
import { localInput, utcFromLocal } from "../../lib/time";
export function ScheduleForm({
  schedule,
  onClose,
}: {
  schedule?: Schedule;
  onClose: () => void;
}) {
  const { data, clis, terminals, perform, busy, setError } = useAppStore();
  const defaults = data!.settings;
  const [name, setName] = useState(schedule?.name ?? "");
  const [cli, setCli] = useState<CliKind>(
    schedule?.cli ?? defaults.default_cli,
  );
  const [terminal, setTerminal] = useState(
    schedule?.terminal.id ??
      terminals.find((t) => t.kind === defaults.default_terminal && t.supported)
        ?.id ??
      "",
  );
  const [directory, setDirectory] = useState(
    schedule?.working_directory ?? defaults.default_directory,
  );
  const [prompt, setPrompt] = useState(schedule?.prompt ?? "");
  const [time, setTime] = useState(
    schedule
      ? localInput(new Date(schedule.scheduled_at))
      : localInput(new Date(Date.now() + 3600_000)),
  );
  const [keepOpen, setKeepOpen] = useState(
    schedule?.keep_open ?? defaults.keep_open,
  );
  const availableTerminals =
    schedule && !terminals.some((t) => t.id === schedule.terminal.id)
      ? [...terminals, schedule.terminal]
      : terminals;
  const selectedCli = clis.find((c) => c.kind === cli);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const selected = availableTerminals.find((t) => t.id === terminal);
      if (!selected?.supported)
        throw new Error("지원되는 터미널을 선택하세요.");
      if (!selectedCli?.available)
        throw new Error("CLI를 설치하거나 설정에서 경로를 지정하세요.");
      const ok = await perform("save_schedule", {
        id: schedule?.id ?? null,
        input: {
          name,
          cli,
          cli_path: selectedCli.executable_path,
          terminal: selected,
          working_directory: directory,
          prompt,
          scheduled_at: utcFromLocal(time),
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          local_time: time,
          keep_open: keepOpen,
        },
      });
      if (ok) onClose();
    } catch (e) {
      setError(String(e));
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
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
            />
          </label>
          <div className="grid">
            <label>
              AI CLI
              <select
                value={cli}
                onChange={(e) => setCli(e.target.value as CliKind)}
              >
                {(["codex", "claude", "gemini"] as const).map((k) => (
                  <option key={k} value={k}>
                    {k}
                    {clis.find((c) => c.kind === k)?.available
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
                value={terminal}
                onChange={(e) => setTerminal(e.target.value)}
              >
                <option value="">터미널 선택</option>
                {availableTerminals.map((t) => (
                  <option key={t.id} value={t.id} disabled={!t.supported}>
                    {t.name}
                    {!t.supported ? " (지원 불가)" : ""}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <label>
            작업 디렉터리
            <div className="row">
              <input
                required
                value={directory}
                onChange={(e) => setDirectory(e.target.value)}
              />
              <button
                type="button"
                onClick={async () => {
                  try {
                    const p = await open({ directory: true, multiple: false });
                    if (typeof p === "string") setDirectory(p);
                  } catch (e) {
                    setError(String(e));
                  }
                }}
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
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
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
              value={time}
              onChange={(e) =>
                setTime(
                  e.target.value.length === 16
                    ? e.target.value + ":00"
                    : e.target.value,
                )
              }
            />
          </label>
          <label className="check">
            <input
              type="checkbox"
              checked={keepOpen}
              onChange={(e) => setKeepOpen(e.target.checked)}
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
