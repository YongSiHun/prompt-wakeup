import { useState, type FormEvent } from "react";
import { Modal, ModalCancelButton } from "../../components/modal";
import { DirectoryField } from "../../components/directory_field";
import { useCliPicker } from "../../hooks/use_cli_picker";
import { useAppStore } from "../../stores/app_store";
import type { Schedule } from "../../types";
import { CliSelect } from "../ai_clis/cli_select";
import { CliEditor } from "../ai_clis/cli_editor";
import { availableClis } from "../ai_clis/cli_service";
import { ScheduleTimeField } from "./schedule_time_field";
import {
  initialDraft,
  buildInput,
  nextScheduleNumber,
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
  const [addingCli, setAddingCli] = useState(false);
  const [draft, setDraft] = useState(() => {
    const initial = initialDraft(schedule, settings);
    if (!schedule) {
      const data = useAppStore.getState().data!;
      initial.name = "새 예약 작업 " + nextScheduleNumber(data.executions);
    }
    if (
      !schedule &&
      !availableClis(clis).some((cli) => cli.kind === initial.cli)
    )
      initial.cli = "";
    return initial;
  });
  const update = <K extends keyof ScheduleDraft>(
    key: K,
    value: ScheduleDraft[K],
  ) => setDraft((current) => ({ ...current, [key]: value }));
  async function submit(event: FormEvent) {
    event.preventDefault();
    const store = useAppStore.getState();
    try {
      const input = buildInput(draft, clis);
      input.name = input.name.trim();
      if (
        store.data!.schedules.some(
          (item) => item.id !== schedule?.id && item.name.trim() === input.name,
        )
      )
        throw new Error(
          "이미 사용 중인 예약 작업 이름입니다. 다른 이름을 입력하세요.",
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
    <Modal
      title={schedule ? "예약 수정" : "새 프롬프트 예약"}
      dismissDisabled={busy}
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
        <label>
          AI CLI
          <CliSelect
            required
            disabled={busy}
            clis={clis}
            value={draft.cli}
            path={draft.cliPath}
            onChange={(cli) =>
              setDraft((current) => ({ ...current, cli, cliPath: "" }))
            }
          />
        </label>
        <div className="executable-path">
          <button
            type="button"
            disabled={busy}
            onClick={async () => {
              if (!draft.cli) {
                setAddingCli(true);
                return;
              }
              const cli = await chooseCli(draft.cli);
              if (cli) update("cliPath", cli.executable_path);
            }}
          >
            AI CLI 실행 파일 선택
          </button>
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
            onInvalid={(event) =>
              event.currentTarget.setCustomValidity("프롬프트를 작성하세요")
            }
            onInput={(event) => event.currentTarget.setCustomValidity("")}
            value={draft.prompt}
            onChange={(event) => update("prompt", event.target.value)}
            placeholder="예약 시간에 CLI가 처리할 작업을 입력하세요."
          />
        </label>
        <ScheduleTimeField
          value={draft.time}
          onChange={(time) => update("time", time)}
        />

        <p className="hint">
          프롬프트는 이 PC의 사용자 데이터 폴더에 저장됩니다. 절전 중 지난
          작업은 자동 실행하지 않습니다.
        </p>
        <footer>
          <ModalCancelButton disabled={busy}>취소</ModalCancelButton>
          <button className="primary" disabled={busy}>
            예약 저장
          </button>
        </footer>
      </form>
      {addingCli && (
        <CliEditor
          onClose={() => setAddingCli(false)}
          onSaved={(cli) =>
            setDraft((current) => ({
              ...current,
              cli: cli.kind,
              cliPath: cli.executable_path,
            }))
          }
        />
      )}
    </Modal>
  );
}
