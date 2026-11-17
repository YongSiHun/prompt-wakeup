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
