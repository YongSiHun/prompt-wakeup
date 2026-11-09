import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot, CliInfo, TerminalInfo } from "../types";
interface Store {
  data: Snapshot | null;
  clis: CliInfo[];
  terminals: TerminalInfo[];
  error: string | null;
  busy: boolean;
  refresh: () => Promise<void>;
  detect: () => Promise<void>;
  perform: (
    command: string,
    args?: Record<string, unknown>,
  ) => Promise<boolean>;
  setError: (error: string | null) => void;
  addTerminal: (terminal: TerminalInfo) => void;
  addCli: (cli: CliInfo) => void;
}
let pendingOperations = 0;
let snapshotRequest = 0;
let detectionRequest: Promise<void> | null = null;
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
async function refresh() {
  const request = ++snapshotRequest;
  try {
    const data = await invoke<Snapshot>("snapshot");
    if (request === snapshotRequest) useAppStore.setState({ data });
  } catch (error) {
    reportError(error);
  }
}
function reportError(error: unknown) {
  useAppStore.setState({ error: String(error) });
}
// Keep manual selections made while a slower automatic scan was in flight.
function mergeDetected<T>(
  detected: T[],
  previous: T[],
  current: T[],
  key: (item: T) => string,
): T[] {
  const before = new Map(previous.map((item) => [key(item), item]));
  const selected = current.filter((item) => before.get(key(item)) !== item);
  return [
    ...new Map(
      [...detected, ...selected].map((item) => [key(item), item]),
    ).values(),
  ];
}
async function discover() {
  const previous = useAppStore.getState();
  try {
    const [clis, terminals] = await Promise.all([
      invoke<CliInfo[]>("detect_clis"),
      invoke<TerminalInfo[]>("detect_terminals"),
    ]);
    useAppStore.setState((current) => ({
      clis: mergeDetected(clis, previous.clis, current.clis, (cli) => cli.kind),
      terminals: mergeDetected(
        terminals,
        previous.terminals,
        current.terminals,
        (terminal) => terminal.id,
      ),
    }));
  } catch (error) {
    reportError(error);
  }
}
function detect() {
  detectionRequest ??= busy(discover).finally(() => {
    detectionRequest = null;
  });
  return detectionRequest;
}
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
  terminals: [],
  error: null,
  busy: false,
  refresh,
  detect,
  perform,
  setError: (error) => set({ error }),
  addTerminal: (terminal) =>
    set((state) => ({
      terminals: [
        ...state.terminals.filter((item) => item.id !== terminal.id),
        terminal,
      ],
    })),
  addCli: (cli) =>
    set((state) => ({
      clis: [...state.clis.filter((item) => item.kind !== cli.kind), cli],
    })),
}));
