import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { StatusBadge } from "../../components/status_badge";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
import type { ExecutionRecord } from "../../types";

export function ExecutionList() {
  const records = useAppStore((state) => state.data!.executions);
  return (
    <section className="execution-list">
      <h2>터미널 / 실행 결과</h2>
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

function useExecutionOutput(record: ExecutionRecord) {
  const [output, setOutput] = useState("");
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    function publish(text: string | null, failure: string | null) {
      if (disposed) return;
      if (text !== null) setOutput(text);
      setError(failure);
    }
    async function refresh() {
      try {
        const text = await invoke<string>("execution_output", {
          id: record.id,
        });
        publish(text || "", null);
      } catch (cause) {
        publish(null, String(cause));
      } finally {
        if (!disposed && !record.ended_at)
          timer = setTimeout(() => void refresh(), 1000);
      }
    }
    void refresh();
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [record.id, record.ended_at, record.status]);
  return { output, error };
}

function ExecutionTerminal({ record }: { record: ExecutionRecord }) {
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

function ExecutionFile({ record }: { record: ExecutionRecord }) {
  const [path, setPath] = useState("");
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    invoke<string>("execution_output_path", { id: record.id }).then(
      (value) => {
        if (!disposed) setPath(value);
      },
      (cause) => {
        if (!disposed) setError(String(cause));
      },
    );
    return () => {
      disposed = true;
    };
  }, [record.id]);
  async function open(folder: boolean) {
    try {
      await invoke("open_execution_output", { id: record.id, folder });
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  }
  return (
    <div className="execution-file">
      <div>
        <small>저장 위치 · UTF-8 텍스트 (.output)</small>
        <code aria-label={record.name + " 출력 파일 경로"}>
          {path || "저장 위치를 불러오는 중…"}
        </code>
      </div>
      <div className="actions">
        <button
          disabled={!path}
          onClick={() => void open(false)}
          aria-label={record.name + " 출력 파일 열기"}
        >
          파일 열기
        </button>
        <button
          disabled={!path}
          onClick={() => void open(true)}
          aria-label={record.name + " 저장 폴더 열기"}
        >
          폴더 열기
        </button>
      </div>
      {error && <p className="hint">파일 확인 실패: {error}</p>}
    </div>
  );
}
