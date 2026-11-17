import { DirectoryField } from "../../components/directory_field";
import { CliCatalog } from "../ai_clis/cli_catalog";
import { useState } from "react";
import { useAppStore } from "../../stores/app_store";
import { cliKinds, type Settings, type CliKind } from "../../types";
export function SettingsPanel() {
  const store = useAppStore.getState();
  const clis = useAppStore((state) => state.clis);
  const busy = useAppStore((state) => state.busy);
  const [settings, setSettings] = useState<Settings>({
    ...store.data!.settings,
  });
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setSettings((current) => ({ ...current, [key]: value }));
  return (
    <section className="settings">
      <section className="settings-section">
        <div className="section-heading">
          <h2>기본 설정</h2>
        </div>
        <div className="grid">
          <label>
            기본 AI CLI
            <select
              value={settings.default_cli}
              onChange={(event) =>
                update("default_cli", event.target.value as CliKind)
              }
            >
              {cliKinds.map((kind) => (
                <option key={kind}>{kind}</option>
              ))}
            </select>
          </label>
          <p className="hint">
            모든 작업은 앱 내 터미널에서 실행되며 출력이 자동 저장됩니다.
          </p>
        </div>
        <DirectoryField
          label="기본 작업 폴더"
          value={settings.default_directory}
          onChange={(path) => update("default_directory", path)}
        />
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
        <button
          className="primary"
          disabled={busy}
          onClick={() => void store.perform("save_settings", { settings })}
        >
          설정 저장
        </button>
      </section>
      <CliCatalog
        clis={clis}
        busy={busy}
        onDetect={() => void store.detect()}
      />
    </section>
  );
}
