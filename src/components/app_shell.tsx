import type { ReactNode } from "react";
import { useEffect } from "react";
import { useAppStore } from "../stores/app_store";
export const pages = {
  schedules: { label: "예약 작업" },
  executions: { label: "실행이력" },
  settings: { label: "설정" },
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
  const count = useAppStore(
    (state) =>
      state.data?.schedules.reduce(
        (total, schedule) => total + Number(schedule.status === "pending"),
        0,
      ) ?? 0,
  );
  return (
    <div className="shell">
      <aside>
        <nav>
          {Object.entries(pages).map(([id, entry]) => (
            <button
              key={id}
              className={page === id ? "selected" : ""}
              onClick={() => navigate(id as Page)}
            >
              {entry.label}
              {id === "schedules" && <span className="nav-count">{count}</span>}
            </button>
          ))}
        </nav>
      </aside>
      <main>
        <Alerts />
        {children}
      </main>
    </div>
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
