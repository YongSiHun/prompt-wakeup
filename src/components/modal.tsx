import { useAppStore } from "../stores/app_store";
import { useEffect, useRef, type ReactNode } from "react";

export function Modal({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const error = useAppStore((state) => state.error);
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="modal"
      aria-modal="true"
      aria-label={title}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <header>
        <h2>{title}</h2>
        <button type="button" aria-label="닫기" onClick={onClose}>
          ×
        </button>
      </header>
      {error && (
        <p className="error" aria-live="polite">
          {error}
        </p>
      )}
      {children}
    </dialog>
  );
}
