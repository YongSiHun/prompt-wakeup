import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../../stores/app_store";

export function NotificationPanel({ enabled }: { enabled: boolean }) {
  const [permission, setPermission] = useState("확인하지 않음");
  const busy = useAppStore((state) => state.busy);
  const store = useAppStore.getState();
  return (
    <section className="settings-section">
      <div className="section-heading">
        <h2>알림 및 권한</h2>
      </div>
      <p className="hint">
        예약 작업의 완료·실패 알림을 받으려면 OS 알림 권한이 필요합니다. 알림을
        받지 않을 경우 권한을 요청할 필요가 없습니다.
      </p>
      <p className="hint" role="status">
        알림 권한: {permission}
      </p>
      <button
        disabled={busy || !enabled}
        onClick={async () => {
          try {
            const result = await invoke<string>("notification_permission", {
              request: true,
            });
            const labels: Record<string, string> = {
              Granted: "허용됨",
              Denied: "허용되지 않음",
              Prompt: "권한 요청 필요",
            };
            setPermission(labels[result] ?? result);
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
