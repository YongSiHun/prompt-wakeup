import { useAppStore } from "../../stores/app_store";
import { ExecutionTerminal } from "./execution_terminal";

export function ExecutionList() {
  const records = useAppStore((state) => state.data!.executions);
  return (
    <section className="execution-list">
      <p className="hint">
        각 작업의 출력과 오류를 실시간으로 표시합니다. 실행이 끝나도 보존 기간
        동안 다시 확인할 수 있습니다.
      </p>
      {records.length === 0 ? (
        <div className="empty">아직 실행 이력이 없습니다.</div>
      ) : (
        [...records]
          .reverse()
          .map((record) => (
            <ExecutionTerminal key={record.id} record={record} />
          ))
      )}
    </section>
  );
}
