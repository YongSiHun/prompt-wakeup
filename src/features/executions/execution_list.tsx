import { useMemo, useState } from "react";
import { useAppStore } from "../../stores/app_store";
import { ExecutionTerminal } from "./execution_terminal";

const PAGE_SIZE = 20;

// UI: 실행이력 페이지.
export function ExecutionList() {
  const records = useAppStore((state) => state.data!.executions);
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE);
  const visible = useMemo(
    () => records.slice(-visibleCount).reverse(),
    [records, visibleCount],
  );
  return (
    <section className="execution-list">
      <header className="page-heading">
        <p className="hint">
          각 작업의 출력과 오류를 실시간으로 표시합니다. 실행이 끝나도 보존 기간
          동안 다시 확인할 수 있습니다.
        </p>
      </header>
      {records.length === 0 && (
        <div className="empty">아직 실행 이력이 없습니다.</div>
      )}
      {visible.map((record) => (
        <ExecutionTerminal key={record.id} record={record} />
      ))}
      {records.length > visibleCount && (
        <button onClick={() => setVisibleCount((count) => count + PAGE_SIZE)}>
          이전 이력 더 보기
        </button>
      )}
    </section>
  );
}
