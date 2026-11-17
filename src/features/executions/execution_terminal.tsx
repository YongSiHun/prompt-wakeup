import { useEffect, useRef } from "react";
import { StatusBadge } from "../../components/status_badge";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
import type { ExecutionRecord } from "../../types";
import { useExecutionOutput } from "./use_execution_output";
import { ExecutionFile } from "./execution_file";

export function ExecutionTerminal({ record }: { record: ExecutionRecord }) {
  const { output, error } = useExecutionOutput(record);
  const viewport = useRef<HTMLPreElement>(null);
  const follow = useRef(true);
  const busy = useAppStore((state) => state.busy);
  const perform = useAppStore((state) => state.perform);
  useEffect(() => {
    if (follow.current && viewport.current)
      viewport.current.scrollTop = viewport.current.scrollHeight;
  }, [output]);
  return (
    <article
      className="execution-terminal"
      aria-label={record.name + " 터미널"}
    >
      <header className="terminal-heading">
        <div>
          <h3>{record.name}</h3>
          <small>
            {record.cli} · {formatTime(record.started_at)}
          </small>
        </div>
        <div className="actions">
          <StatusBadge status={record.status} />
          <button
            className="danger"
            disabled={busy || !record.ended_at}
            title={
              !record.ended_at
                ? "실행이 끝난 후 삭제할 수 있습니다."
                : undefined
            }
            aria-label={record.name + " 실행 결과 삭제"}
            onClick={() => void perform("delete_execution", { id: record.id })}
          >
            삭제
          </button>
        </div>
      </header>
      <pre
        ref={viewport}
        className="terminal-output"
        tabIndex={0}
        aria-label={record.name + " 출력"}
        onScroll={(event) => {
          const view = event.currentTarget;
          follow.current =
            view.scrollHeight - view.scrollTop - view.clientHeight < 32;
        }}
      >
        {output ||
          (record.ended_at
            ? "저장된 출력이 없습니다. 이전 버전에서 실행한 작업은 출력을 제공하지 않습니다."
            : "출력을 기다리는 중입니다…")}
      </pre>
      {error && (
        <p role="alert" className="error">
          출력을 불러오지 못했습니다: {error}
        </p>
      )}
      <ExecutionFile record={record} />
      <footer>
        {record.ended_at ? formatTime(record.ended_at) : "실행 중"} · 종료 코드:{" "}
        {record.exit_code ?? "—"}
        {record.error && <p>{record.error}</p>}
      </footer>
    </article>
  );
}
