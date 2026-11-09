import { useEffect } from "react";
import { listen, type UnlistenFn, type Event } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/app_store";
type Subscription = readonly [string, (event: Event<unknown>) => void];
function subscriptions(navigate: (page: string) => void): Subscription[] {
  const store = useAppStore.getState();
  return [
    ["state-changed", () => void store.refresh()],
    ["backend-error", (event) => store.setError(String(event.payload))],
    ["navigate", (event) => navigate(String(event.payload))],
  ];
}
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
function subscribeAll(events: Subscription[]) {
  let disposed = false;
  const cleanup: UnlistenFn[] = [];
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
export function useBackend(navigate: (page: string) => void) {
  useEffect(() => {
    const store = useAppStore.getState();
    void store.refresh();
    void store.detect();
    return subscribeAll(subscriptions(navigate));
  }, [navigate]);
}
