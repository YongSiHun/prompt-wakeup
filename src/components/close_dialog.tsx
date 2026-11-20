import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Modal, ModalCancelButton } from "./modal";
import { useAppStore } from "../stores/app_store";

// UI: 트레이·종료 선택.
export function CloseDialog({ onClose }: { onClose: () => void }) {
  const [busy, setBusy] = useState(false);
  // UI: 닫기 처리.
  async function close(action: "tray" | "quit") {
    setBusy(true);
    try {
      await invoke("close_app", { action });
      onClose();
    } catch (error) {
      useAppStore.getState().setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Modal
      confirmDismiss={false}
      title="앱 닫기"
      dismissDisabled={busy}
      onClose={onClose}
    >
      <p className="hint">
        트레이에 두면 예약 작업이 계속 실행됩니다. 완전히 종료하면 앱을 다시 열
        때까지 예약 작업이 실행되지 않습니다.
      </p>
      <footer>
        <ModalCancelButton disabled={busy}>취소</ModalCancelButton>
        <button
          className="danger"
          disabled={busy}
          onClick={() => void close("quit")}
        >
          완전히 종료
        </button>
        <button
          className="primary"
          disabled={busy}
          onClick={() => void close("tray")}
        >
          트레이로 보내기
        </button>
      </footer>
    </Modal>
  );
}
