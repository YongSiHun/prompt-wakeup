import { beforeEach, describe, it, expect, vi } from "vitest";
import type { Snapshot } from "../src/types";
const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
function snapshot(active: boolean): Snapshot {
  return {
    active,
    warnings: [],
    schedules: [],
    executions: [],
    settings: {
      default_cli: "codex",
      default_terminal: "cmd",
      default_directory: "",
      theme: "system",
      keep_open: true,
      notifications: false,
      retention_days: 90,
      custom_cli_paths: {},
      custom_terminals: [],
    },
  };
}
beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
});
describe("UI 비동기 상태", () => {
  it("동시에 요청한 탐지는 하나의 작업을 공유한다", async () => {
    const pending = deferred<unknown[]>();
    invoke.mockImplementation(() => pending.promise);
    const { useAppStore } = await import("../src/stores/app_store");
    const first = useAppStore.getState().detect();
    const second = useAppStore.getState().detect();
    expect(first).toBe(second);
    expect(invoke).toHaveBeenCalledTimes(2);
    pending.resolve([]);
    await first;
    expect(useAppStore.getState().busy).toBe(false);
  });
  it("하나가 끝나도 다른 명령이 진행 중이면 busy를 유지한다", async () => {
    const first = deferred<void>();
    const second = deferred<void>();
    invoke
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const { useAppStore } = await import("../src/stores/app_store");
    useAppStore.setState({ refresh: async () => {} });
    const one = useAppStore.getState().perform("save_settings");
    const two = useAppStore.getState().perform("save_schedule");
    first.resolve();
    await one;
    expect(useAppStore.getState().busy).toBe(true);
    second.resolve();
    await two;
    expect(useAppStore.getState().busy).toBe(false);
  });
  it("늦게 도착한 이전 조회 결과로 최신 화면을 덮어쓰지 않는다", async () => {
    const first = deferred<Snapshot>();
    const second = deferred<Snapshot>();
    invoke
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const { useAppStore } = await import("../src/stores/app_store");
    const one = useAppStore.getState().refresh();
    const two = useAppStore.getState().refresh();
    const newest = snapshot(false);
    second.resolve(newest);
    await two;
    first.resolve(snapshot(true));
    await one;
    expect(useAppStore.getState().data).toBe(newest);
  });
  it("탐지 실패 후 busy를 해제하고 오류를 표시한다", async () => {
    invoke.mockRejectedValue(new Error("탐지 실패"));
    const { useAppStore } = await import("../src/stores/app_store");
    await useAppStore.getState().detect();
    expect(useAppStore.getState().busy).toBe(false);
    expect(useAppStore.getState().error).toContain("탐지 실패");
  });
});
