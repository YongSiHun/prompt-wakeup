import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot, CliInfo } from "../types";
interface Store {
  data: Snapshot | null;
  clis: CliInfo[];
  error: string | null;
  busy: boolean;
  refresh: () => Promise<void>;
  detect: () => Promise<void>;
  perform: (
    command: string,
    args?: Record<string, unknown>,
  ) => Promise<boolean>;
  setError: (error: string | null) => void;
  addCli: (cli: CliInfo) => void;
}
let pendingOperations = 0;
let snapshotRequest = 0;
let detectionRequest: Promise<void> | null = null;
// 상태: 진행 중 작업 집계.
async function busy<T>(operation: () => Promise<T>): Promise<T> {
  pendingOperations++;
  useAppStore.setState({ busy: true });
  try {
    return await operation();
  } finally {
    pendingOperations--;
    useAppStore.setState({ busy: pendingOperations > 0 });
  }
}
// IPC: 최신 조회 반영.
async function refresh() {
  const request = ++snapshotRequest;
  try {
    const data = await invoke<Snapshot>("snapshot");
    if (request === snapshotRequest) useAppStore.setState({ data });
  } catch (error) {
    reportError(error);
  }
}
// 상태: 오류 표시.
function reportError(error: unknown) {
  useAppStore.setState({ error: String(error) });
}
// Keep manual selections made while a slower automatic scan was in flight.
function mergeDetected(
  detected: CliInfo[],
  previous: CliInfo[],
  current: CliInfo[],
): CliInfo[] {
  const before = new Map(previous.map((item) => [item.kind, item]));
  const selected = current.filter((item) => before.get(item.kind) !== item);
  return [
    ...new Map(
      [...detected, ...selected].map((item) => [item.kind, item]),
    ).values(),
  ];
}
// IPC: CLI 탐지.
async function discover() {
  const previous = useAppStore.getState().clis;
  try {
    const clis = await invoke<CliInfo[]>("detect_clis");
    useAppStore.setState((current) => ({
      clis: mergeDetected(clis, previous, current.clis),
    }));
  } catch (error) {
    reportError(error);
  }
}
// IPC: 중복 탐지 병합.
function detect() {
  detectionRequest ??= busy(discover).finally(() => {
    detectionRequest = null;
  });
  return detectionRequest;
}
// IPC: 명령 실행·상태 갱신.
async function perform(
  command: string,
  args?: Record<string, unknown>,
): Promise<boolean> {
  return busy(async () => {
    useAppStore.setState({ error: null });
    try {
      await invoke(command, args);
      await useAppStore.getState().refresh();
      return true;
    } catch (error) {
      reportError(error);
      return false;
    }
  });
}
export const useAppStore = create<Store>((set) => ({
  data: null,
  clis: [],
  error: null,
  busy: false,
  refresh,
  detect,
  perform,
  // 상태: 공통 오류 갱신.
  setError: (error) => set({ error }),
  // CLI: 같은 종류의 경로 교체.
  addCli: (cli) =>
    set((state) => ({
      clis: [...state.clis.filter((item) => item.kind !== cli.kind), cli],
    })),
}));
