import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ExecutionRecord } from "../../types";

export function ExecutionFile({ record }: { record: ExecutionRecord }) {
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
