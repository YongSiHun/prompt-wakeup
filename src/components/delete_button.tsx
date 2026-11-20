import { useState } from "react";
import { Modal, ModalCancelButton } from "./modal";
import { useAppStore } from "../stores/app_store";

export function DeleteButton({
  name,
  description,
  disabled,
  title,
  onDelete,
}: {
  name: string;
  description: string;
  disabled?: boolean;
  title?: string;
  onDelete: () => Promise<boolean>;
}) {
  const [confirming, setConfirming] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const close = () => {
    if (!deleting) setConfirming(false);
  };
  async function remove() {
    setDeleting(true);
    try {
      if (await onDelete()) setConfirming(false);
    } catch (error) {
      useAppStore.getState().setError(String(error));
    } finally {
      setDeleting(false);
    }
  }
  return (
    <>
      <button
        className="danger"
        disabled={disabled}
        title={title}
        aria-label={name + " 삭제"}
        onClick={() => setConfirming(true)}
      >
        삭제
      </button>
      {confirming && (
        <Modal
          confirmDismiss={false}
          title="삭제하시겠습니까?"
          onClose={close}
          dismissDisabled={deleting}
        >
          <p>{name}</p>
          <p className="hint">{description}</p>
          <footer>
            <ModalCancelButton disabled={deleting} autoFocus>
              취소
            </ModalCancelButton>
            <button
              className="danger"
              disabled={disabled || deleting}
              onClick={() => void remove()}
            >
              {deleting ? "삭제 중…" : "삭제"}
            </button>
          </footer>
        </Modal>
      )}
    </>
  );
}
