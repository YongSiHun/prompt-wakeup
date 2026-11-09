import { TerminalCatalog } from "../terminals/terminal_catalog";
import { CliCatalog } from "../ai_clis/cli_catalog";
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/app_store";
import type { Settings, CliKind, CliInfo, TerminalInfo } from "../../types";
export function SettingsPanel() {
  const store = useAppStore();
  const [permission, setPermission] = useState("확인하지 않음");
  const [settings, setSettings] = useState<Settings>({
    ...store.data!.settings,
  });
  const update = <K extends keyof Settings>(k: K, v: Settings[K]) =>
    setSettings((s) => ({ ...s, [k]: v }));
  async function manual(kind: "terminal" | CliKind) {
    try {
      const path = await open({ multiple: false, directory: false });
      if (typeof path !== "string") return;
      if (kind === "terminal")
        store.addTerminal(
          await invoke<TerminalInfo>("manual_terminal", { path }),
        );
      else store.addCli(await invoke<CliInfo>("manual_cli", { kind, path }));
      await store.refresh();
      setSettings({ ...useAppStore.getState().data!.settings });
    } catch (e) {
      store.setError(String(e));
    }
  }
  return (
    <section className="settings">
      <h2>설정</h2>
      <div className="grid">
        <label>
          기본 AI CLI
          <select
            value={settings.default_cli}
            onChange={(e) => update("default_cli", e.target.value as CliKind)}
          >
            {["codex", "claude", "gemini"].map((x) => (
              <option key={x}>{x}</option>
            ))}
          </select>
        </label>
        <label>
          기본 터미널
          <select
            value={settings.default_terminal}
            onChange={(e) => update("default_terminal", e.target.value)}
          >
            {store.terminals
              .filter((t) => t.supported)
              .map((t) => (
                <option key={t.id} value={t.kind}>
                  {t.name}
                </option>
              ))}
          </select>
        </label>
      </div>
      <label>
        기본 작업 폴더
        <div className="row">
          <input
            value={settings.default_directory}
            onChange={(e) => update("default_directory", e.target.value)}
          />
          <button
            onClick={async () => {
              const p = await open({ directory: true });
              if (typeof p === "string") update("default_directory", p);
            }}
          >
            폴더 선택
          </button>
        </div>
      </label>
      <div className="grid">
        <label>
          테마
          <select
            value={settings.theme}
            onChange={(e) =>
              update("theme", e.target.value as Settings["theme"])
            }
          >
            <option value="system">시스템</option>
            <option value="light">라이트</option>
            <option value="dark">다크</option>
          </select>
        </label>
        <label>
          이력 보존 기간 (일)
          <input
            type="number"
            min="1"
            max="3650"
            value={settings.retention_days}
            onChange={(e) => update("retention_days", Number(e.target.value))}
          />
        </label>
      </div>
      <label className="check">
        <input
          type="checkbox"
          checked={settings.keep_open}
          onChange={(e) => update("keep_open", e.target.checked)}
        />
        기본 터미널 창 유지
      </label>
      <label className="check">
        <input
          type="checkbox"
          checked={settings.notifications}
          onChange={(e) => update("notifications", e.target.checked)}
        />
        시스템 알림 사용
      </label>
      <button
        className="primary"
        disabled={store.busy}
        onClick={() => void store.perform("save_settings", { settings })}
      >
        설정 저장
      </button>
      <CliCatalog
        clis={store.clis}
        busy={store.busy}
        onDetect={() => void store.detect()}
        onManual={(kind) => void manual(kind)}
      />
      <TerminalCatalog
        terminals={store.terminals}
        onManual={() => void manual("terminal")}
      />
      <h3>권한 안내</h3>
      <p className="hint">알림 권한: {permission}</p>
      <button
        onClick={async () => {
          try {
            setPermission(
              (await invoke<string>("notification_permission", {
                request: true,
              })) +
                " · 플러그인 결과이며 실제 OS 알림 허용은 시스템 설정에서 확인하세요.",
            );
          } catch (e) {
            store.setError(String(e));
          }
        }}
      >
        알림 권한 확인 / 요청
      </button>
      <p className="hint">
        관리자 권한은 필요하지 않습니다. CLI의 기존 로그인 및 승인 정책을
        따릅니다. 인증·사용량·네트워크·권한 오류는 외부 터미널에서 확인 후 수동
        재실행하세요.
      </p>
      <p className="hint">
        macOS: 터미널 자동화는 최초 OS 승인이 필요합니다. 거부했다면 시스템 설정
        → 개인정보 보호 및 보안 → 자동화를 확인하세요. 알림은 OS 알림 설정에
        따라 제한될 수 있습니다. 현재 권한 상태를 자동으로 판정하지 않습니다.
      </p>
    </section>
  );
}
