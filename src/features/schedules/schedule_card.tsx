import { ConfirmButton } from "../../components/confirm_button";
import { memo } from "react";
import { useAppStore } from "../../stores/app_store";
import { useNow } from "../../hooks/use_now";
import { StatusBadge } from "../../components/status_badge";
import { formatTime, remaining } from "../../lib/time";
import type { Schedule } from "../../types";
type CardProps = {
  schedule: Schedule;
  onEdit: (schedule: Schedule) => void;
};
// UI: 대기 예약.
export const ScheduleCard = memo(function ScheduleCard({
  schedule,
  onEdit,
}: CardProps) {
  return (
    <article className="schedule-card">
      <div className="card-title">
        <h3>{schedule.name}</h3>
        <StatusBadge status={schedule.status} />
      </div>
      <div className="meta">
        <span>{schedule.cli.toUpperCase()}</span>
        <span title={schedule.working_directory}>
          {schedule.working_directory}
        </span>
      </div>
      <div className="card-bottom">
        <div>
          <time>{formatTime(schedule.scheduled_at)}</time>
          <Countdown value={schedule.scheduled_at} />
        </div>
        <ScheduleActions schedule={schedule} onEdit={onEdit} />
      </div>
    </article>
  );
});
// UI: 남은 시간.
function Countdown({ value }: { value: string }) {
  const now = useNow();
  return <small>{remaining(value, now)}</small>;
}
// UI: 예약 수정·실행·취소.
function ScheduleActions({ schedule, onEdit }: CardProps) {
  const busy = useAppStore((state) => state.busy);
  return (
    <div className="actions">
      <button disabled={busy} onClick={() => onEdit(schedule)}>
        수정
      </button>
      <button disabled={busy} onClick={() => void runNow(schedule.id)}>
        지금 실행
      </button>
      <ConfirmButton
        label="취소"
        dialogTitle="예약을 취소할까요?"
        confirmLabel="예약 취소"
        cancelLabel="돌아가기"
        name={schedule.name}
        description="예약을 취소하면 정해진 시각에 프롬프트가 실행되지 않습니다."
        disabled={busy}
        onConfirm={() =>
          useAppStore
            .getState()
            .perform("schedule_action", { id: schedule.id, action: "cancel" })
        }
      />
    </div>
  );
}
// IPC: 확인 후 즉시 실행.
async function runNow(id: string) {
  if (
    window.confirm(
      "앱 내 터미널에서 이 작업을 지금 실행할까요? 결과 미확인 작업은 기존 CLI가 중단되었는지 확인하세요.",
    )
  )
    await useAppStore
      .getState()
      .perform("schedule_action", { id, action: "run" });
}
