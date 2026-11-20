import { useAppStore } from "../../stores/app_store";
import type { ExecutionRecord } from "../../types";

// UI: 실행 당시 프롬프트.
export function ExecutionPrompt({ record }: { record: ExecutionRecord }) {
  const savedPrompt = useAppStore(
    (state) =>
      state.data?.schedules.find(
        (schedule) => schedule.id === record.schedule_id,
      )?.prompt,
  );
  const recordedPrompt = record.prompt ?? null;
  const prompt = recordedPrompt ?? savedPrompt;
  return (
    <section className="execution-prompt">
      <h4>
        {recordedPrompt !== null
          ? "실행 당시 프롬프트"
          : "예약에 저장된 프롬프트"}
      </h4>
      {recordedPrompt === null && (
        <p className="hint">
          이 실행에는 당시 프롬프트가 저장되지 않았습니다.
          {savedPrompt !== undefined &&
            " 현재 예약의 내용이며, 실행 이후 수정되었을 수 있습니다."}
        </p>
      )}
      {prompt !== undefined && (
        <pre tabIndex={0} aria-label={record.name + " 프롬프트"}>
          {prompt}
        </pre>
      )}
    </section>
  );
}
