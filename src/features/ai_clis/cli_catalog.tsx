import type { CliInfo, CliKind } from "../../types";
export function CliCatalog({
  clis,
  busy,
  onDetect,
  onManual,
}: {
  clis: CliInfo[];
  busy: boolean;
  onDetect: () => void;
  onManual: (kind: CliKind) => void;
}) {
  return (
    <>
      <h3>CLI 탐지</h3>
      <button disabled={busy} onClick={onDetect}>
        CLI / 터미널 재탐색
      </button>
      {clis.map((c) => (
        <div className="detect" key={c.kind}>
          <strong>{c.kind}</strong>
          <span>{c.version ?? c.validation_error}</span>
          <code>{c.executable_path}</code>
          <button onClick={() => onManual(c.kind)}>실행 파일 지정</button>
        </div>
      ))}
    </>
  );
}
