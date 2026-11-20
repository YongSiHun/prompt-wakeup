import { Modal, ModalCancelButton } from "../../components/modal";
import { memo, useState } from "react";
import { useAppStore } from "../../stores/app_store";
import { useNow } from "../../hooks/use_now";
import { StatusBadge } from "../../components/status_badge";
import { formatTime, remaining } from "../../lib/time";
import type { Schedule } from "../../types";
type CardProps = {
  schedule: Schedule;
  onEdit: (schedule: Schedule) => void;
};
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
          <ScheduleTime schedule={schedule} />
        </div>
        <ScheduleActions schedule={schedule} onEdit={onEdit} />
      </div>
    </article>
  );
});
function ScheduleTime({ schedule }: { schedule: Schedule }) {
  if (schedule.status === "pending")
    return <Countdown value={schedule.scheduled_at} />;
  return null;
}
function Countdown({ value }: { value: string }) {
  const now = useNow();
  return <small>{remaining(value, now)}</small>;
}
function ScheduleActions({ schedule, onEdit }: CardProps) {
  return (
    <div className="actions">
      <PendingActions schedule={schedule} onEdit={onEdit} />
      <IdleActions schedule={schedule} />
    </div>
  );
}
function PendingActions({
  schedule,
  onEdit,
}: Pick<CardProps, "schedule" | "onEdit">) {
  if (schedule.status !== "pending") return null;
  return (
    <>
      <button onClick={() => onEdit(schedule)}>수정</button>
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
      <CancelScheduleButton schedule={schedule} disabled={busy} />
    </>
  );
}

function CancelScheduleButton({
  schedule,
  disabled,
}: {
  schedule: Schedule;
  disabled: boolean;
}) {
  const [confirming, setConfirming] = useState(false);
  const [canceling, setCanceling] = useState(false);
  if (schedule.status !== "pending") return null;
  async function cancel() {
    setCanceling(true);
    try {
      if (
        await useAppStore
          .getState()
          .perform("schedule_action", { id: schedule.id, action: "cancel" })
      )
        setConfirming(false);
    } catch (error) {
      useAppStore.getState().setError(String(error));
    } finally {
      setCanceling(false);
    }
  }
  return (
    <>
      <button
        className="danger"
        disabled={disabled || canceling}
        onClick={() => setConfirming(true)}
      >
        취소
      </button>
      {confirming && (
        <Modal
          title="예약을 취소할까요?"
          confirmDismiss={false}
          dismissDisabled={disabled || canceling}
          onClose={() => setConfirming(false)}
        >
          <p>
            <strong>이 예약 작업을 정말 취소할까요?</strong>
          </p>
          <p>{schedule.name}</p>
          <p className="hint">
            예약을 취소하면 정해진 시각에 프롬프트가 실행되지 않습니다.
          </p>
          <footer>
            <ModalCancelButton disabled={disabled || canceling} autoFocus>
              돌아가기
            </ModalCancelButton>
            <button
              className="danger"
              disabled={disabled || canceling}
              onClick={() => void cancel()}
            >
              {canceling ? "취소 중…" : "예약 취소"}
            </button>
          </footer>
        </Modal>
      )}
    </>
  );
}
