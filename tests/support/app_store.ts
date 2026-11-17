import { useAppStore } from "../../src/stores/app_store";
import { makeSnapshot } from "./fixtures";
type AppState = ReturnType<typeof useAppStore.getState>;
export function resetAppStore(overrides: Partial<AppState> = {}) {
  useAppStore.setState(
    { ...useAppStore.getInitialState(), data: makeSnapshot(), ...overrides },
    true,
  );
}
