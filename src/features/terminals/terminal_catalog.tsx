import type { TerminalInfo } from "../../types";
export function TerminalCatalog({
  terminals,
  busy,
  onManual,
}: {
  terminals: TerminalInfo[];
  busy: boolean;
  onManual: () => void;
}) {
  return (
    <>
      <h3>터미널 탐지</h3>
      <button disabled={busy} onClick={onManual}>
        터미널 수동 선택
      </button>
      {terminals.map((terminal) => (
        <div className="detect" key={terminal.id}>
          <strong>
            {terminal.name} · {terminal.supported ? "실행 지원" : "사용 불가"}
          </strong>
          <code>{terminal.executable_path}</code>
          <span>{terminal.validation_error}</span>
        </div>
      ))}
    </>
  );
}
