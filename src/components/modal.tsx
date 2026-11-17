import { useAppStore } from "../stores/app_store";
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  type ReactNode,
  type ButtonHTMLAttributes,
} from "react";
const DismissContext = createContext<() => void>(() => {});
export function ModalCancelButton(
  props: ButtonHTMLAttributes<HTMLButtonElement>,
) {
  const dismiss = useContext(DismissContext);
  return <button {...props} type="button" onClick={dismiss} />;
}
export function Modal({
  title,
  onClose,
  children,
  dismissDisabled = false,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
  dismissDisabled?: boolean;
}) {
  const error = useAppStore((state) => state.error);
  const dialog = useRef<HTMLDialogElement>(null);
  const requestDismiss = () => {
    if (
      !dismissDisabled &&
      window.confirm("이 창을 닫을까요? 저장하지 않은 내용은 사라집니다.")
    )
      onClose();
  };
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
        event.stopPropagation();
        requestDismiss();
      }}
      onClick={(event) => {
        if (event.target !== event.currentTarget) return;
        const rect = event.currentTarget.getBoundingClientRect();
        if (
          event.clientX < rect.left ||
          event.clientX > rect.right ||
          event.clientY < rect.top ||
          event.clientY > rect.bottom
        )
          requestDismiss();
      }}
    >
      {error && (
        <p className="error" aria-live="polite">
          {error}
        </p>
      )}
      <DismissContext.Provider value={requestDismiss}>
        {children}
      </DismissContext.Provider>
    </dialog>
  );
}
