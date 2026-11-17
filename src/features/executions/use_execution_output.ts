import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ExecutionRecord } from "../../types";

export function useExecutionOutput(record: ExecutionRecord) {
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
