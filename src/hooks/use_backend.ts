import { useEffect } from "react";
import { listen, type UnlistenFn, type Event } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/app_store";
type Subscription = readonly [string, (event: Event<unknown>) => void];
// IPC: 백엔드 이벤트 정의.
function subscriptions(
  navigate: (page: string) => void,
  requestClose: () => void,
): Subscription[] {
  const store = useAppStore.getState();
  return [
    ["state-changed", () => void store.refresh()],
    ["backend-error", (event) => store.setError(String(event.payload))],
    ["navigate", (event) => navigate(String(event.payload))],
    ["close-requested", requestClose],
  ];
}
// IPC: 이벤트 등록.
async function subscribeOne(
  [name, handler]: Subscription,
  register: (cleanup: UnlistenFn) => void,
) {
  try {
    register(await listen(name, handler));
  } catch (error) {
    useAppStore.getState().setError(String(error));
  }
}
// IPC: 이벤트 구독·해제.
function subscribeAll(events: Subscription[]) {
  let disposed = false;
  const cleanup: UnlistenFn[] = [];
  // 상태: 구독 해제 함수 보관.
  function register(unlisten: UnlistenFn) {
    if (disposed) {
      unlisten();
      return;
    }
    cleanup.push(unlisten);
  }
  events.forEach((event) => void subscribeOne(event, register));
  return () => {
    disposed = true;
    cleanup.forEach((unlisten) => unlisten());
  };
}
// IPC: 초기 조회·이벤트 구독.
export function useBackend(
  navigate: (page: string) => void,
  requestClose: () => void,
) {
  useEffect(() => {
    const store = useAppStore.getState();
    void store.refresh();
    void store.detect();
    return subscribeAll(subscriptions(navigate, requestClose));
  }, [navigate, requestClose]);
}
