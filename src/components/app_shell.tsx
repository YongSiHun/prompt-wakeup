import type { ReactNode } from "react";
import { useEffect } from "react";
import { useAppStore } from "../stores/app_store";
export const pages = {
  schedules: { label: "예약 작업", title: "다음 작업을 준비하세요" },
  executions: { label: "실행 이력", title: "실행 기록" },
  settings: { label: "설정", title: "나의 실행 환경" },
  status: { label: "스케줄러 상태", title: "스케줄러 상태" },
};
export type Page = keyof typeof pages;
export function isPage(value: string): value is Page {
  return Object.hasOwn(pages, value);
}
export function AppShell({
  page,
  navigate,
  children,
}: {
  page: Page;
  navigate: (page: Page) => void;
  children: ReactNode;
}) {
  const theme = useAppStore((state) => state.data?.settings.theme);
  useEffect(() => {
    document.documentElement.dataset.theme = theme ?? "system";
  }, [theme]);
  return (
    <div className="shell">
      <aside>
        <div className="brand">
          <span className="logo">◷</span>
          <strong>PromptWakeup</strong>
        </div>
        <p className="slogan">Wake up your prompts on time.</p>
        <nav>
          {Object.entries(pages).map(([id, entry]) => (
            <button
              key={id}
              className={page === id ? "selected" : ""}
              onClick={() => navigate(id as Page)}
            >
              {entry.label}
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
        <AppHeader page={page} />
        <Alerts />
        {children}
      </main>
    </div>
  );
}
function AppHeader({ page }: { page: Page }) {
  const active = useAppStore((state) => state.data?.active);
  return (
    <header className="top">
      <div>
        <span className="eyebrow">YOUR PROMPTS, ON SCHEDULE</span>
        <h1>{pages[page].title}</h1>
      </div>
      <span className="live">
        <span className="dot" />
        {active ? "스케줄러 활성" : "연결 확인 중"}
      </span>
    </header>
  );
}
function Alerts() {
  const error = useAppStore((state) => state.error);
  const warnings = useAppStore((state) => state.data?.warnings);
  const setError = useAppStore((state) => state.setError);
  return (
    <>
      {error && (
        <div role="alert" className="error">
          {error}
          <button onClick={() => setError(null)}>닫기</button>
        </div>
      )}
      {warnings?.map((warning) => (
        <p key={warning} className="error">
          {warning}
        </p>
      ))}
    </>
  );
}
