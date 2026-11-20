import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { registerCli } from "../features/ai_clis/cli_service";
import { useAppStore } from "../stores/app_store";
import type { CliKind } from "../types";
// I/O: CLI 선택·등록.
export function useCliPicker() {
  const [picking, setPicking] = useState(false);
  // I/O: CLI 선택·검증.
  async function chooseCli(kind: CliKind) {
    const store = useAppStore.getState();
    setPicking(true);
    store.setError(null);
    try {
      const path = await open({
        title: kind + " 실행 파일 선택",
        multiple: false,
        directory: false,
      });
      return typeof path === "string" ? await registerCli(kind, path) : null;
    } catch (error) {
      store.setError(String(error));
      return null;
    } finally {
      setPicking(false);
    }
  }
  return { picking, chooseCli };
}
