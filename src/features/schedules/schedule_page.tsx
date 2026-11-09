import { useCallback, useMemo, useState } from "react";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
import { ScheduleCard } from "./schedule_card";
import { ScheduleForm } from "./schedule_form";
import type { Schedule } from "../../types";
type Editor = Schedule | "new" | null;
export function SchedulePage({ showStatus = false }: { showStatus?: boolean }) {
  const [editor, setEditor] = useState<Editor>(null);
  const [detail, setDetail] = useState<Schedule | null>(null);
  const create = useCallback(() => setEditor("new"), []);
  const edit = useCallback((schedule: Schedule) => setEditor(schedule), []);
  return (
    <>
      <ScheduleSummary />
      {showStatus && (
        <div className="notice">
          절전 복귀·시스템 시간 변경으로 놓친 예약은 자동 실행하지 않습니다.
          실행 중인 터미널은 앱 완전 종료 후에도 계속 작업할 수 있으며, 결과는
          다음 시작 시 반영합니다.
        </div>
      )}
      <ScheduleList onCreate={create} onEdit={edit} onDetail={setDetail} />
      <p className="hint">
        사용량 제한 해제 시각은 직접 입력합니다. 예약 실패 시 자동 재시도하지
        않습니다.
      </p>
      <ScheduleEditor editor={editor} onClose={() => setEditor(null)} />
      <ScheduleDetail schedule={detail} onClose={() => setDetail(null)} />
    </>
  );
}
export function StatusPage() {
  return <SchedulePage showStatus />;
}
function summarize(schedules: Schedule[]) {
  const pending = schedules.filter((schedule) => schedule.status === "pending");
  const next = pending.reduce<Schedule | undefined>(
    (current, schedule) => earlier(current, schedule),
    undefined,
  );
  return { count: pending.length, next };
}
function earlier(current: Schedule | undefined, candidate: Schedule) {
  if (!current) return candidate;
  return candidate.scheduled_at < current.scheduled_at ? candidate : current;
}
function ScheduleSummary() {
  const schedules = useAppStore((state) => state.data!.schedules);
  const summary = useMemo(() => summarize(schedules), [schedules]);
  const next = summary.next;
  return (
    <div className="stats">
      <section>
        <small>다음 예약</small>
        <strong>
          {next ? formatTime(next.scheduled_at) : "예정된 작업 없음"}
        </strong>
      </section>
      <section>
        <small>대기 중인 예약</small>
        <strong>
          {summary.count}
          <em>개</em>
        </strong>
      </section>
      <section>
        <small>실행 환경</small>
        <strong>외부 터미널</strong>
      </section>
    </div>
  );
}
type ListProps = {
  onCreate: () => void;
  onEdit: (schedule: Schedule) => void;
  onDetail: (schedule: Schedule) => void;
};
function ScheduleList({ onCreate, onEdit, onDetail }: ListProps) {
  const schedules = useAppStore((state) => state.data!.schedules);
  const sorted = useMemo(
    () =>
      [...schedules].sort((a, b) =>
        a.scheduled_at.localeCompare(b.scheduled_at),
      ),
    [schedules],
  );
  return (
    <>
      <div className="section-heading">
        <h2>
          예약 작업 <small>{schedules.length}</small>
        </h2>
        <button className="primary" onClick={onCreate}>
          + 새 예약
        </button>
      </div>
      {sorted.length === 0 ? (
        <EmptySchedules onCreate={onCreate} />
      ) : (
        <div className="schedule-list">
          {sorted.map((schedule) => (
            <ScheduleCard
              key={schedule.id}
              schedule={schedule}
              onEdit={onEdit}
              onDetail={onDetail}
            />
          ))}
        </div>
      )}
    </>
  );
}
function EmptySchedules({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="empty">
      <span className="empty-icon">◷</span>
      <h3>깨울 프롬프트를 예약해 보세요</h3>
      <p>작업 폴더와 프롬프트를 정하고, 실행할 시간을 선택하세요.</p>
      <button onClick={onCreate}>첫 예약 만들기</button>
    </div>
  );
}
function ScheduleEditor({
  editor,
  onClose,
}: {
  editor: Editor;
  onClose: () => void;
}) {
  if (!editor) return null;
  return (
    <ScheduleForm
      schedule={editor === "new" ? undefined : editor}
      onClose={onClose}
    />
  );
}
function ScheduleDetail({
  schedule,
  onClose,
}: {
  schedule: Schedule | null;
  onClose: () => void;
}) {
  if (!schedule) return null;
  return (
    <div className="overlay">
      <section
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-label="예약 상세"
      >
        <header>
          <h2>{schedule.name}</h2>
          <button onClick={onClose}>닫기</button>
        </header>
        <p>
          {schedule.cli} · {schedule.terminal.name}
        </p>
        <p>{schedule.working_directory}</p>
        <p>
          {formatTime(schedule.scheduled_at)} · {schedule.timezone}
        </p>
        <pre>{schedule.prompt}</pre>
      </section>
    </div>
  );
}
