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
  addTerminal: (t: TerminalInfo) => void;
  addCli: (c: CliInfo) => void;
}
export const useAppStore = create<Store>((set, get) => ({
  data: null,
  clis: [],
  terminals: [],
  error: null,
  busy: false,
  setError: (error) => set({ error }),
  addTerminal: (t) =>
    set((s) => ({
      terminals: [...s.terminals.filter((x) => x.id !== t.id), t],
    })),
  addCli: (c) =>
    set((s) => ({ clis: [...s.clis.filter((x) => x.kind !== c.kind), c] })),
  refresh: async () => {
    try {
      set({ data: await invoke<Snapshot>("snapshot") });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  detect: async () => {
    set({ busy: true });
    try {
      const [clis, terminals] = await Promise.all([
        invoke<CliInfo[]>("detect_clis"),
        invoke<TerminalInfo[]>("detect_terminals"),
      ]);
      set({ clis, terminals });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ busy: false });
    }
  },
  perform: async (command, args) => {
    set({ busy: true, error: null });
    try {
      await invoke(command, args);
      await get().refresh();
      return true;
    } catch (e) {
      set({ error: String(e) });
      return false;
    } finally {
      set({ busy: false });
    }
  },
}));
