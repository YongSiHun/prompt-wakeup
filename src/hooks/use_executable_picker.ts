import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../stores/app_store";
import type { CliInfo, CliKind, TerminalInfo } from "../types";

export function useExecutablePicker() {
  const [picking, setPicking] = useState(false);
  async function pick<T>(
    title: string,
    command: string,
    args: Record<string, unknown>,
    apply: (value: T) => void,
  ) {
    const store = useAppStore.getState();
    setPicking(true);
    store.setError(null);
    try {
      const path = await open({ title, multiple: false, directory: false });
      if (typeof path !== "string") return null;
      const value = await invoke<T>(command, { ...args, path });
      apply(value);
      await store.refresh();
      return value;
    } catch (error) {
      store.setError(String(error));
      return null;
    } finally {
      setPicking(false);
    }
  }
  return {
    picking,
    chooseTerminal: () =>
      pick<TerminalInfo>(
        "터미널 실행 파일 선택",
        "manual_terminal",
        {},
        useAppStore.getState().addTerminal,
      ),
    chooseCli: (kind: CliKind) =>
      pick<CliInfo>(
        kind + " 실행 파일 선택",
        "manual_cli",
        { kind },
        useAppStore.getState().addCli,
      ),
  };
}
