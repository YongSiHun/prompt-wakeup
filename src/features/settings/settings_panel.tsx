import { CliCatalog } from "../ai_clis/cli_catalog";
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/app_store";
import type { Settings, CliKind } from "../../types";
import { useExecutablePicker } from "../../hooks/use_executable_picker";
export function SettingsPanel() {
  const store = useAppStore.getState();
  const clis = useAppStore((state) => state.clis);
  const { picking, chooseCli } = useExecutablePicker();
  const busy = useAppStore((state) => state.busy) || picking;
  const [permission, setPermission] = useState("확인하지 않음");
  const [settings, setSettings] = useState<Settings>({
    ...store.data!.settings,
  });
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setSettings((current) => ({ ...current, [key]: value }));
  return (
    <section className="settings">
      <h2>설정</h2>
      <div className="grid">
        <label>
          기본 AI CLI
          <select
            value={settings.default_cli}
            onChange={(event) =>
              update("default_cli", event.target.value as CliKind)
            }
          >
            {["codex", "claude", "gemini"].map((kind) => (
              <option key={kind}>{kind}</option>
            ))}
          </select>
        </label>
        <p className="hint">
          모든 작업은 앱 내 터미널에서 실행되며 출력이 자동 저장됩니다.
        </p>
      </div>
      <label>
        기본 작업 폴더
        <div className="row">
          <input
            value={settings.default_directory}
            onChange={(event) =>
              update("default_directory", event.target.value)
            }
          />
          <button
            onClick={async () => {
              try {
                const path = await open({ directory: true, multiple: false });
                if (typeof path === "string") update("default_directory", path);
              } catch (error) {
                store.setError(String(error));
              }
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
            onChange={(event) =>
              update("theme", event.target.value as Settings["theme"])
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
            onChange={(event) =>
              update("retention_days", Number(event.target.value))
            }
          />
        </label>
      </div>
      <label className="check">
        <input
          type="checkbox"
          checked={settings.notifications}
          onChange={(event) => update("notifications", event.target.checked)}
        />
        시스템 알림 사용
      </label>
      <button
        className="primary"
        disabled={busy}
        onClick={() => void store.perform("save_settings", { settings })}
      >
        설정 저장
      </button>
      <CliCatalog
        clis={clis}
        busy={busy}
        onDetect={() => void store.detect()}
        onManual={async (kind) => {
          if (await chooseCli(kind)) update("default_cli", kind);
        }}
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
        따릅니다. 인증·사용량·네트워크·권한 오류는 앱 내 터미널에서 확인 후 수동
        재실행하세요.
      </p>
      <p className="hint">알림은 OS 알림 설정에 따라 제한될 수 있습니다.</p>
    </section>
  );
}
