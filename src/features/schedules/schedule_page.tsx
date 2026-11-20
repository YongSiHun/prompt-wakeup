import { useCallback, useMemo, useState } from "react";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
import { ScheduleCard } from "./schedule_card";
import { ScheduleForm } from "./schedule_form";
import type { Schedule } from "../../types";
type Editor = Schedule | "new" | null;
// UI: 대기 예약 화면.
export function SchedulePage() {
  const schedules = useAppStore((state) => state.data!.schedules);
  const pending = useMemo(
    () =>
      schedules
        .filter((schedule) => schedule.status === "pending")
        .sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at)),
    [schedules],
  );
  const [editor, setEditor] = useState<Editor>(null);
  // UI: 새 예약 열기.
  const create = useCallback(() => setEditor("new"), []);
  // UI: 예약 수정 열기.
  const edit = useCallback((schedule: Schedule) => setEditor(schedule), []);
  return (
    <>
      <ScheduleSummary schedules={pending} />
      <ScheduleList schedules={pending} onCreate={create} onEdit={edit} />
      <p className="hint">
        사용량 제한 해제 시각은 직접 입력합니다. 예약 실패 시 자동 재시도하지
        않습니다.
      </p>
      {editor && (
        <ScheduleForm
          schedule={editor === "new" ? undefined : editor}
          onClose={() => setEditor(null)}
        />
      )}
    </>
  );
}
// UI: 다음 예약·개수.
function ScheduleSummary({ schedules }: { schedules: Schedule[] }) {
  const next = schedules[0];
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
          {schedules.length}
          <em>개</em>
        </strong>
      </section>
    </div>
  );
}
type ListProps = {
  schedules: Schedule[];
  onCreate: () => void;
  onEdit: (schedule: Schedule) => void;
};
// UI: 예약 목록.
function ScheduleList({ schedules, onCreate, onEdit }: ListProps) {
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
      {schedules.length === 0 ? (
        <EmptySchedules onCreate={onCreate} />
      ) : (
        <div className="schedule-list">
          {schedules.map((schedule) => (
            <ScheduleCard
              key={schedule.id}
              schedule={schedule}
              onEdit={onEdit}
            />
          ))}
        </div>
      )}
    </>
  );
}
// UI: 빈 예약 안내.
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
