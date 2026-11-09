import { useCallback, useState } from "react";
import { AppShell, isPage, type Page } from "./components/app_shell";
import { useBackend } from "./hooks/use_backend";
import { useAppStore } from "./stores/app_store";
import { SchedulePage, StatusPage } from "./features/schedules/schedule_page";
import { ExecutionList } from "./features/executions/execution_list";
import { SettingsPanel } from "./features/settings/settings_panel";
const screens = {
  schedules: SchedulePage,
  status: StatusPage,
  executions: ExecutionList,
  settings: SettingsPanel,
};
export default function App() {
  const [page, setPage] = useState<Page>("schedules");
  const navigate = useCallback((value: string) => {
    if (isPage(value)) setPage(value);
  }, []);
  useBackend(navigate);
  return (
    <AppShell page={page} navigate={setPage}>
      <Screen page={page} />
    </AppShell>
  );
}
function Screen({ page }: { page: Page }) {
  const ready = useAppStore((state) => state.data !== null);
  if (!ready)
    return (
      <div className="empty">
        로컬 예약 정보를 불러오는 중입니다. Tauri 데스크톱 앱에서 실행하세요.
      </div>
    );
  const Content = screens[page];
  return <Content />;
}
