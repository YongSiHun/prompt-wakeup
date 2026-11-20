import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { Modal, ModalCancelButton } from "../../src/components/modal";
import { resetAppStore } from "../support/app_store";
beforeEach(() => resetAppStore());
it.each(["취소", "Escape", "배경"])(
  "%s로 닫을 때 확인하고 거절하면 유지한다",
  (method) => {
    const close = vi.fn();
    render(
      <Modal title="편집" onClose={close}>
        <ModalCancelButton>취소</ModalCancelButton>
      </Modal>,
    );
    const dialog = screen.getByRole("dialog");
    const dismiss = () => {
      if (method === "취소")
        fireEvent.click(screen.getByRole("button", { name: "취소" }));
      else if (method === "Escape")
        fireEvent(
          dialog,
          new Event("cancel", { bubbles: true, cancelable: true }),
        );
      else fireEvent.click(dialog, { clientX: -10, clientY: -10 });
    };
    dismiss();
    expect(screen.getByText(/저장하지 않은 내용은 사라집니다/)).toBeTruthy();
    expect(close).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "계속 작성" }));
    dismiss();
    fireEvent.click(screen.getByRole("button", { name: "저장하지 않고 닫기" }));
    expect(close).toHaveBeenCalledOnce();
    expect(screen.queryByRole("heading")).toBeNull();
    expect(screen.queryByRole("button", { name: "닫기" })).toBeNull();
  },
);
it("처리 중에는 닫기 확인과 닫기를 실행하지 않는다", () => {
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
  const close = vi.fn();
  render(
    <Modal title="편집" onClose={close} dismissDisabled>
      <ModalCancelButton>취소</ModalCancelButton>
    </Modal>,
  );
  fireEvent.click(screen.getByRole("button", { name: "취소" }));
  fireEvent(
    screen.getByRole("dialog"),
    new Event("cancel", { cancelable: true }),
  );
  expect(confirm).not.toHaveBeenCalled();
  expect(close).not.toHaveBeenCalled();
});
