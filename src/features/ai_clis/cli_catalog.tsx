import { useState, type FormEvent } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { Modal } from "../../components/modal";
import { useAppStore } from "../../stores/app_store";
import type { CliInfo, CliKind } from "../../types";

export function CliCatalog({
  clis,
  busy,
  onDetect,
}: {
  clis: CliInfo[];
  busy: boolean;
  onDetect: () => void;
}) {
  const [editing, setEditing] = useState<CliInfo | "new" | null>(null);
  return (
    <section className="settings-section">
      <div className="section-heading">
        <h2>CLI 탐지</h2>
        <div className="actions">
          <button disabled={busy} onClick={onDetect}>
            CLI 재탐색
          </button>
          <button
            className="primary"
            disabled={busy}
            onClick={() => setEditing("new")}
          >
            + 추가
          </button>
        </div>
      </div>
      <p className="hint">
        사용할 CLI 이름과 실행 파일 경로를 직접 설정할 수 있습니다.
      </p>
      {clis.map((cli) => (
        <CliRow key={cli.kind} cli={cli} busy={busy} onEdit={setEditing} />
      ))}
      {editing && (
        <CliEditor
          cli={editing === "new" ? undefined : editing}
          onClose={() => setEditing(null)}
        />
      )}
    </section>
  );
}

function CliRow({
  cli,
  busy,
  onEdit,
}: {
  cli: CliInfo;
  busy: boolean;
  onEdit: (cli: CliInfo) => void;
}) {
  return (
    <article className="detect">
      <div className="card-title">
        <h3>{cli.kind}</h3>
        <span className={"badge " + (cli.available ? "completed" : "failed")}>
          {cli.available ? "사용 가능" : "확인 필요"}
        </span>
      </div>
      <p className="hint">
        {cli.version ?? cli.validation_error ?? "탐지되지 않음"}
      </p>
      <code>{cli.executable_path || "설정된 경로 없음"}</code>
      <div className="actions">
        <button disabled={busy} onClick={() => onEdit(cli)}>
          수정
        </button>
      </div>
    </article>
  );
}

function CliEditor({ cli, onClose }: { cli?: CliInfo; onClose: () => void }) {
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
      const info = await invoke<CliInfo>("manual_cli", {
        kind,
        path: path.trim(),
      });
      store.addCli(info);
      await store.refresh();
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
            {["codex", "claude", "gemini"].map((name) => (
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
