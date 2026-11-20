import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { ConfirmButton } from "../../src/components/confirm_button";
import { resetAppStore } from "../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => resetAppStore());
it("취소하면 삭제하지 않는다", () => {
  vi.spyOn(window, "confirm").mockReturnValue(true);
  const remove = vi.fn().mockResolvedValue(true);
  render(
    <ConfirmButton
      label="삭제"
      dialogTitle="삭제하시겠습니까?"
      ariaLabel="작업 삭제"
      name="작업"
      description="저장 파일도 삭제합니다."
      onConfirm={remove}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "작업 삭제" }));
  const dialog = screen.getByRole("dialog", { name: "삭제하시겠습니까?" });
  expect(remove).not.toHaveBeenCalled();
  fireEvent.click(within(dialog).getByRole("button", { name: "취소" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(remove).not.toHaveBeenCalled();
});
it("삭제에 실패하면 확인창을 유지한다", async () => {
  const remove = vi.fn().mockResolvedValue(false);
  render(
    <ConfirmButton
      label="삭제"
      dialogTitle="삭제하시겠습니까?"
      ariaLabel="작업 삭제"
      name="작업"
      description="저장 파일도 삭제합니다."
      onConfirm={remove}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "작업 삭제" }));
  fireEvent.click(
    within(screen.getByRole("dialog")).getByRole("button", { name: "삭제" }),
  );
  await waitFor(() => expect(remove).toHaveBeenCalledOnce());
  expect(screen.getByRole("dialog")).toBeTruthy();
});
