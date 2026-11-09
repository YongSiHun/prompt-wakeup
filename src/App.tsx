import { StatusBadge } from "./components/status_badge";
import { useState, useCallback, useEffect } from "react";
import { useAppStore } from "./stores/app_store";
import { useBackend } from "./hooks/use_backend";
import { ScheduleForm } from "./features/schedules/schedule_form";
import { ExecutionList } from "./features/executions/execution_list";
import { SettingsPanel } from "./features/settings/settings_panel";
import { formatTime, remaining, labels } from "./lib/time";
import type { Schedule } from "./types";
export default function App() {
  const [page, setPage] = useState("schedules");
  const navigate = useCallback((p: string) => setPage(p), []);
  useBackend(navigate);
  const { data, error, setError, perform, busy } = useAppStore();
  const [form, setForm] = useState<Schedule | "new" | null>(null);
  const [detail, setDetail] = useState<Schedule | null>(null);
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  useEffect(() => {
    if (data) document.documentElement.dataset.theme = data.settings.theme;
  }, [data?.settings.theme]);
  const pending = data?.schedules.filter((s) => s.status === "pending") ?? [];
  const next = [...pending].sort((a, b) =>
    a.scheduled_at.localeCompare(b.scheduled_at),
  )[0];
  return (
    <div className="shell">
      <aside>
        <div className="brand">
          <span className="logo">◷</span>
          <strong>PromptWakeup</strong>
        </div>
        <p className="slogan">Wake up your prompts on time.</p>
        <nav>
          {[
            ["schedules", "예약 작업"],
            ["executions", "실행 이력"],
            ["settings", "설정"],
            ["status", "스케줄러 상태"],
          ].map(([id, title]) => (
            <button
              className={page === id ? "selected" : ""}
              key={id}
              onClick={() => setPage(id)}
            >
              {title}
            </button>
          ))}
        </nav>
        <div className="aside-note">
          <span className="dot" />앱 실행 중에만 예약 유효
          <br />
          닫기 버튼을 누르면 트레이로 숨깁니다.
        </div>
      </aside>
      <main>
        <header className="top">
          <div>
            <span className="eyebrow">YOUR PROMPTS, ON SCHEDULE</span>
            <h1>
              {page === "executions"
                ? "실행 기록"
                : page === "settings"
                  ? "나의 실행 환경"
                  : page === "status"
                    ? "스케줄러 상태"
                    : "다음 작업을 준비하세요"}
            </h1>
          </div>
          <span className="live">
            <span className="dot" />
            {data?.active ? "스케줄러 활성" : "연결 확인 중"}
          </span>
        </header>
        {error && (
          <div role="alert" className="error">
            {error}
            <button onClick={() => setError(null)}>닫기</button>
          </div>
        )}
        {data?.warnings.map((w) => (
          <p className="error" key={w}>
            {w}
          </p>
        ))}
        {!data ? (
          <div className="empty">
            로컬 예약 정보를 불러오는 중입니다. Tauri 데스크톱 앱에서
            실행하세요.
          </div>
        ) : page === "settings" ? (
          <SettingsPanel />
        ) : page === "executions" ? (
          <ExecutionList />
        ) : (
          <>
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
                  {pending.length}
                  <em>개</em>
                </strong>
              </section>
              <section>
                <small>실행 환경</small>
                <strong>외부 터미널</strong>
              </section>
            </div>
            {page === "status" && (
              <div className="notice">
                스케줄러는 Rust 백엔드에서 실행됩니다. 절전 복귀·시스템 시간
                변경으로 놓친 예약은 자동 실행하지 않습니다. 실행 중인 터미널은
                앱 완전 종료 후에도 계속 작업할 수 있으며, 결과는 다음 시작 시
                반영합니다.
              </div>
            )}
            <div className="section-heading">
              <h2>
                예약 작업 <small>{data.schedules.length}</small>
              </h2>
              <button className="primary" onClick={() => setForm("new")}>
                + 새 예약
              </button>
            </div>
            {data.schedules.length === 0 ? (
              <div className="empty">
                <span className="empty-icon">◷</span>
                <h3>깨울 프롬프트를 예약해 보세요</h3>
                <p>작업 폴더와 프롬프트를 정하고, 실행할 시간을 선택하세요.</p>
                <button onClick={() => setForm("new")}>첫 예약 만들기</button>
              </div>
            ) : (
              <div className="schedule-list">
                {[...data.schedules]
                  .sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at))
                  .map((s) => (
                    <article className="schedule-card" key={s.id}>
                      <div className="card-title">
                        <h3>{s.name}</h3>
                        <StatusBadge status={s.status} />
                      </div>
                      <div className="meta">
                        <span>{s.cli.toUpperCase()}</span>
                        <span>{s.terminal.name}</span>
                        <span title={s.working_directory}>
                          {s.working_directory}
                        </span>
                      </div>
                      <div className="card-bottom">
                        <div>
                          <time>{formatTime(s.scheduled_at)}</time>
                          <small>
                            {s.status === "pending"
                              ? remaining(s.scheduled_at, now)
                              : s.timezone}
                          </small>
                        </div>
                        <div className="actions">
                          <button onClick={() => setDetail(s)}>상세</button>
                          {s.status === "pending" && (
                            <>
                              <button onClick={() => setForm(s)}>수정</button>
                              <button
                                disabled={busy}
                                onClick={() =>
                                  void perform("schedule_action", {
                                    id: s.id,
                                    action: "cancel",
                                  })
                                }
                              >
                                취소
                              </button>
                            </>
                          )}
                          {!["launching", "launched"].includes(s.status) && (
                            <>
                              <button
                                disabled={busy}
                                onClick={() => {
                                  if (
                                    window.confirm(
                                      "외부 터미널에서 이 작업을 지금 실행할까요?",
                                    )
                                  )
                                    void perform("schedule_action", {
                                      id: s.id,
                                      action: "run",
                                    });
                                }}
                              >
                                {s.status === "failed" ? "재실행" : "지금 실행"}
                              </button>
                              <button
                                className="danger"
                                disabled={busy}
                                onClick={() => {
                                  if (
                                    window.confirm(
                                      "예약을 삭제할까요? 실행 이력은 보존합니다.",
                                    )
                                  )
                                    void perform("schedule_action", {
                                      id: s.id,
                                      action: "delete",
                                    });
                                }}
                              >
                                삭제
                              </button>
                            </>
                          )}
                        </div>
                      </div>
                    </article>
                  ))}
              </div>
            )}
            <p className="hint">
              사용량 제한 해제 시각은 직접 입력합니다. 예약 실패 시 자동
              재시도하지 않습니다.
            </p>
          </>
        )}
        {form && data && (
          <ScheduleForm
            schedule={form === "new" ? undefined : form}
            onClose={() => setForm(null)}
          />
        )}{" "}
        {detail && (
          <div className="overlay">
            <section
              className="modal"
              role="dialog"
              aria-modal="true"
              aria-label="예약 상세"
            >
              <header>
                <h2>{detail.name}</h2>
                <button onClick={() => setDetail(null)}>닫기</button>
              </header>
              <p>
                {detail.cli} · {detail.terminal.name}
              </p>
              <p>{detail.working_directory}</p>
              <p>
                {formatTime(detail.scheduled_at)} · {detail.timezone}
              </p>
              <pre>{detail.prompt}</pre>
            </section>
          </div>
        )}
      </main>
    </div>
  );
}
