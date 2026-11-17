import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { CloseDialog } from "../../src/components/close_dialog";
import { resetAppStore } from "../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  vi.mocked(invoke).mockReset();
  resetAppStore();
});
it.each([
  ["트레이로 보내기", "tray"],
  ["완전히 종료", "quit"],
])("%s 선택을 백엔드에 전달한다", async (label, action) => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  const onClose = vi.fn();
  render(<CloseDialog onClose={onClose} />);
  fireEvent.click(screen.getByRole("button", { name: label }));
  await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  expect(invoke).toHaveBeenCalledWith("close_app", { action });
});
it("취소하면 앱을 닫지 않는다", () => {
  vi.spyOn(window, "confirm").mockReturnValue(true);
  const onClose = vi.fn();
  render(<CloseDialog onClose={onClose} />);
  fireEvent.click(screen.getByRole("button", { name: "취소" }));
  expect(onClose).toHaveBeenCalledOnce();
  expect(invoke).not.toHaveBeenCalled();
});
it("종료 실패 시 선택 창과 오류를 유지한다", async () => {
  vi.mocked(invoke).mockRejectedValue("저장 실패");
  const onClose = vi.fn();
  render(<CloseDialog onClose={onClose} />);
  fireEvent.click(screen.getByRole("button", { name: "완전히 종료" }));
  await waitFor(() => expect(screen.getByText("저장 실패")).toBeTruthy());
  expect(onClose).not.toHaveBeenCalled();
});
