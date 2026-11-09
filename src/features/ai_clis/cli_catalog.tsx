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
      {(["codex", "claude", "gemini"] as const).map((kind) => {
        const cli = clis.find((candidate) => candidate.kind === kind);
        return (
          <div className="detect" key={kind}>
            <strong>{kind}</strong>
            <span>
              {cli?.version ?? cli?.validation_error ?? "탐지되지 않음"}
            </span>
            <code>{cli?.executable_path}</code>
            <button disabled={busy} onClick={() => onManual(kind)}>
              실행 파일 지정
            </button>
          </div>
        );
      })}
    </>
  );
}
