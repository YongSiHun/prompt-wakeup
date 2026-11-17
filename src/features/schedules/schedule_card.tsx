import { DeleteButton } from "../../components/delete_button";
import { memo } from "react";
import { useAppStore } from "../../stores/app_store";
import { useNow } from "../../hooks/use_now";
import { StatusBadge } from "../../components/status_badge";
import { formatTime, remaining } from "../../lib/time";
import type { Schedule } from "../../types";
type CardProps = {
  schedule: Schedule;
  onDetail: (schedule: Schedule) => void;
  onEdit: (schedule: Schedule) => void;
};
export const ScheduleCard = memo(function ScheduleCard({
  schedule,
  onDetail,
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
          <ScheduleTime schedule={schedule} />
        </div>
        <ScheduleActions
          schedule={schedule}
          onDetail={onDetail}
          onEdit={onEdit}
        />
      </div>
    </article>
  );
});
function ScheduleTime({ schedule }: { schedule: Schedule }) {
  if (schedule.status === "pending")
    return <Countdown value={schedule.scheduled_at} />;
  return <small>{schedule.timezone}</small>;
}
function Countdown({ value }: { value: string }) {
  const now = useNow();
  return <small>{remaining(value, now)}</small>;
}
function ScheduleActions({ schedule, onDetail, onEdit }: CardProps) {
  return (
    <div className="actions">
      <button onClick={() => onDetail(schedule)}>상세</button>
      <PendingActions schedule={schedule} onEdit={onEdit} />
      <IdleActions schedule={schedule} />
    </div>
  );
}
function PendingActions({
  schedule,
  onEdit,
}: Pick<CardProps, "schedule" | "onEdit">) {
  const busy = useAppStore((state) => state.busy);
  const perform = useAppStore((state) => state.perform);
  if (schedule.status !== "pending") return null;
  return (
    <>
      <button onClick={() => onEdit(schedule)}>수정</button>
      <button
        disabled={busy}
        onClick={() =>
          void perform("schedule_action", { id: schedule.id, action: "cancel" })
        }
      >
        취소
      </button>
    </>
  );
}
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
function IdleActions({ schedule }: { schedule: Schedule }) {
  const busy = useAppStore((state) => state.busy);
  if (["launching", "launched"].includes(schedule.status)) return null;
  return (
    <>
      <button disabled={busy} onClick={() => void runNow(schedule.id)}>
        {schedule.status === "failed" ? "재실행" : "지금 실행"}
      </button>
      <DeleteButton
        name={schedule.name + " 예약"}
        description="예약을 삭제합니다. 실행 이력은 보존됩니다."
        disabled={busy}
        onDelete={() =>
          useAppStore
            .getState()
            .perform("schedule_action", { id: schedule.id, action: "delete" })
        }
      />
    </>
  );
}
