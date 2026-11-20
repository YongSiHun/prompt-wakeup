import { ConfirmButton } from "../../components/confirm_button";
import { useEffect, useRef } from "react";
import { StatusBadge } from "../../components/status_badge";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
import type { ExecutionRecord } from "../../types";
import { useExecutionOutput } from "./use_execution_output";
import { ExecutionFile } from "./execution_file";
import { ExecutionPrompt } from "./execution_prompt";

// UI: 작업 출력·상태.
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
          <div className="execution-title">
            <h3>{record.name}</h3>
            <StatusBadge status={record.status} />
          </div>
          <small>
            {record.cli} · {formatTime(record.started_at)}
          </small>
        </div>
        <div className="actions">
          <ConfirmButton
            label="삭제"
            dialogTitle="삭제하시겠습니까?"
            ariaLabel={record.name + " 실행 결과 삭제"}
            name={record.name + " 실행 결과"}
            description="실행 기록과 저장된 프롬프트·출력 파일을 삭제합니다."
            disabled={busy || !record.ended_at}
            title={
              !record.ended_at
                ? "실행이 끝난 후 삭제할 수 있습니다."
                : undefined
            }
            onConfirm={() => perform("delete_execution", { id: record.id })}
          />
        </div>
      </header>
      <ExecutionPrompt record={record} />
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
        {record.ended_at ? formatTime(record.ended_at) : "실행 중"} · 상태:{" "}
        {executionState(record)}
        {record.error && <p>{record.error}</p>}
      </footer>
    </article>
  );
}

// 상태: 성공·실패·실행중.
function executionState(record: ExecutionRecord): string {
  if (!record.ended_at) return "실행중";
  return record.status === "completed" ? "성공" : "실패";
}
