import { availableClis } from "./cli_service";
import { CliEditor } from "./cli_editor";
import { useState } from "react";
import type { CliInfo } from "../../types";

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
  const detected = availableClis(clis);
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
      {detected.length === 0 && (
        <p className="hint">
          탐지된 CLI가 없습니다. 재탐색하거나 직접 추가하세요.
        </p>
      )}
      {detected.map((cli) => (
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
        <span className="badge completed">사용 가능</span>
      </div>
      {cli.version && <p className="hint">{cli.version}</p>}
      <code>{cli.executable_path}</code>
      <div className="actions">
        <button disabled={busy} onClick={() => onEdit(cli)}>
          수정
        </button>
      </div>
    </article>
  );
}
