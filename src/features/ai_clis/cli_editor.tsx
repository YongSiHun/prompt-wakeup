import { useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Modal } from "../../components/modal";
import { useAppStore } from "../../stores/app_store";
import { cliKinds, type CliInfo, type CliKind } from "../../types";
import { registerCli } from "./cli_service";

export function CliEditor({
  cli,
  onClose,
}: {
  cli?: CliInfo;
  onClose: () => void;
}) {
  const [kind, setKind] = useState<CliKind>(cli?.kind ?? "codex");
  const [path, setPath] = useState(cli?.executable_path ?? "");
  const [busy, setBusy] = useState(false);
  const store = useAppStore.getState();
  async function browse() {
    try {
      const selected = await open({
        title: "CLI 실행 파일 선택",
        multiple: false,
        directory: false,
      });
      if (typeof selected === "string") setPath(selected);
    } catch (error) {
      store.setError(String(error));
    }
  }
  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    store.setError(null);
    try {
      await registerCli(kind, path);
      onClose();
    } catch (error) {
      store.setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Modal
      title={cli ? "CLI 수정" : "CLI 추가"}
      onClose={() => {
        if (!busy) onClose();
      }}
    >
      <form onSubmit={submit}>
        <label>
          CLI 이름
          <select
            value={kind}
            disabled={busy || !!cli}
            onChange={(event) => setKind(event.target.value as CliKind)}
          >
            {cliKinds.map((name) => (
              <option key={name}>{name}</option>
            ))}
          </select>
        </label>
        <label>
          실행 파일 경로
          <div className="row">
            <input
              required
              value={path}
              disabled={busy}
              onChange={(event) => setPath(event.target.value)}
              placeholder="C:\tools\codex.exe"
            />
            <button type="button" disabled={busy} onClick={() => void browse()}>
              파일 선택
            </button>
          </div>
        </label>
        <p className="hint">
          선택한 CLI의 실행 파일과 버전을 확인한 후 저장합니다. 같은 이름이 이미
          있으면 경로를 갱신합니다.
        </p>
        <footer>
          <button type="button" disabled={busy} onClick={onClose}>
            취소
          </button>
          <button className="primary" disabled={busy}>
            {busy ? "확인 중…" : "저장"}
          </button>
        </footer>
      </form>
    </Modal>
  );
}
