import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useAppStore } from "../stores/app_store";
export function useBackend(navigate: (page: string) => void) {
  useEffect(() => {
    const store = useAppStore.getState();
    void store.refresh();
    void store.detect();
    let disposed = false;
    const cleanup: UnlistenFn[] = [];
    const sub = async () => {
      for (const [name, handler] of [
        ["state-changed", () => void store.refresh()],
        [
          "backend-error",
          (e: { payload: unknown }) => store.setError(String(e.payload)),
        ],
        ["navigate", (e: { payload: unknown }) => navigate(String(e.payload))],
      ] as const) {
        try {
          const fn = await listen(name, handler);
          if (disposed) fn();
          else cleanup.push(fn);
        } catch (e) {
          store.setError(String(e));
        }
      }
    };
    void sub();
    return () => {
      disposed = true;
      cleanup.forEach((fn) => fn());
    };
  }, [navigate]);
}
