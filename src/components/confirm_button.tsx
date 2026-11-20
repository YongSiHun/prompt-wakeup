import { useState } from "react";
import { Modal, ModalCancelButton } from "./modal";
import { useAppStore } from "../stores/app_store";

// UI: 비동기 동작 확인·중복 실행 방지.
export function ConfirmButton({
  name,
  description,
  label,
  dialogTitle,
  confirmLabel = label,
  cancelLabel = "취소",
  disabled,
  title,
  ariaLabel,
  onConfirm,
}: {
  name: string;
  description: string;
  label: string;
  dialogTitle: string;
  confirmLabel?: string;
  cancelLabel?: string;
  disabled?: boolean;
  title?: string;
  ariaLabel?: string;
  onConfirm: () => Promise<boolean>;
}) {
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  // IPC: 성공 시 닫고 실패 시 입력 유지.
  async function confirm() {
    setBusy(true);
    try {
      if (await onConfirm()) setConfirming(false);
    } catch (error) {
      useAppStore.getState().setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <button
        className="danger"
        disabled={disabled || busy}
        title={title}
        aria-label={ariaLabel}
        onClick={() => setConfirming(true)}
      >
        {label}
      </button>
      {confirming && (
        <Modal
          title={dialogTitle}
          confirmDismiss={false}
          dismissDisabled={disabled || busy}
          onClose={() => setConfirming(false)}
        >
          <p>{name}</p>
          <p className="hint">{description}</p>
          <footer>
            <ModalCancelButton disabled={disabled || busy} autoFocus>
              {cancelLabel}
            </ModalCancelButton>
            <button
              className="danger"
              disabled={disabled || busy}
              onClick={() => void confirm()}
            >
              {busy ? label + " 중…" : confirmLabel}
            </button>
          </footer>
        </Modal>
      )}
    </>
  );
}
