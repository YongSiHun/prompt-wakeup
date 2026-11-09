import type { TerminalInfo } from "../../types";
export function TerminalCatalog({
  terminals,
  onManual,
}: {
  terminals: TerminalInfo[];
  onManual: () => void;
}) {
  return (
    <>
      <h3>터미널 탐지</h3>
      <button onClick={onManual}>터미널 수동 선택</button>
      {terminals.map((t) => (
        <div className="detect" key={t.id}>
          <strong>
            {t.name} · {t.supported ? "실행 지원" : "사용 불가"}
          </strong>
          <code>{t.executable_path}</code>
          <span>{t.validation_error}</span>
        </div>
      ))}
    </>
  );
}
