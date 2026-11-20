import { useAppStore } from "../stores/app_store";
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
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
  confirmDismiss = true,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
  dismissDisabled?: boolean;
  confirmDismiss?: boolean;
}) {
  const error = useAppStore((state) => state.error);
  const dialog = useRef<HTMLDialogElement>(null);
  const [confirming, setConfirming] = useState(false);
  const requestDismiss = () => {
    if (dismissDisabled) return;
    if (confirmDismiss) setConfirming(true);
    else onClose();
  };
  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);
  return (
    <>
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
      {confirming && (
        <Modal
          title="변경 내용을 버릴까요?"
          confirmDismiss={false}
          onClose={() => setConfirming(false)}
        >
          <p>저장하지 않은 내용은 사라집니다. 이 창을 닫을까요?</p>
          <footer>
            <ModalCancelButton autoFocus>계속 작성</ModalCancelButton>
            <button
              type="button"
              className="danger"
              disabled={dismissDisabled}
              onClick={onClose}
            >
              저장하지 않고 닫기
            </button>
          </footer>
        </Modal>
      )}
    </>
  );
}
