import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/app_store";
import type { CliInfo, CliKind } from "../../types";
export async function registerCli(
  kind: CliKind,
  path: string,
): Promise<CliInfo> {
  const cli = await invoke<CliInfo>("manual_cli", { kind, path: path.trim() });
  const store = useAppStore.getState();
  store.addCli(cli);
  await store.refresh();
  return cli;
}
