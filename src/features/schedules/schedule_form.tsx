import { useState, type FormEvent } from "react";
import { Modal } from "../../components/modal";
import { DirectoryField } from "../../components/directory_field";
import { useCliPicker } from "../../hooks/use_cli_picker";
import { useAppStore } from "../../stores/app_store";
import { cliKinds, type Schedule, type CliKind } from "../../types";
import {
  initialDraft,
  buildInput,
  cliPath,
  normalizeTime,
  type ScheduleDraft,
} from "./schedule_draft";

export function ScheduleForm({
  schedule,
  onClose,
}: {
  schedule?: Schedule;
  onClose: () => void;
}) {
  const settings = useAppStore((state) => state.data!.settings);
  const clis = useAppStore((state) => state.clis);
  const { picking, chooseCli } = useCliPicker();
  const busy = useAppStore((state) => state.busy) || picking;
  const [draft, setDraft] = useState(() => initialDraft(schedule, settings));
  const update = <K extends keyof ScheduleDraft>(
    key: K,
    value: ScheduleDraft[K],
  ) => setDraft((current) => ({ ...current, [key]: value }));
  async function submit(event: FormEvent) {
    event.preventDefault();
    const store = useAppStore.getState();
    try {
      const input = buildInput(draft, clis);
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
    <Modal
      title={schedule ? "예약 수정" : "새 프롬프트 예약"}
      onClose={onClose}
    >
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
              {cliKinds.map((kind) => (
                <option key={kind} value={kind}>
                  {kind}
                  {clis.find((cli) => cli.kind === kind)?.available
                    ? ""
                    : " (사용 불가)"}
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
        </div>
        <DirectoryField
          label="작업 디렉터리"
          required
          value={draft.directory}
          onChange={(path) => update("directory", path)}
        />
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
          실행 날짜 및 시각 · {Intl.DateTimeFormat().resolvedOptions().timeZone}
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
    </Modal>
  );
}
