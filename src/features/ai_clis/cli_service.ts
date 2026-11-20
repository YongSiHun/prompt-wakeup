import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/app_store";
import type { CliInfo, CliKind } from "../../types";
// 검증: 사용 가능한 CLI 필터.
export function availableClis(clis: CliInfo[]) {
  return clis.filter(
    (cli) => cli.available && cli.executable_path.trim() !== "",
  );
}
// IPC: CLI 검증·등록.
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
