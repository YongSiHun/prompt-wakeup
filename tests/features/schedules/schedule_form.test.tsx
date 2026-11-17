import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { ScheduleForm } from "../../../src/features/schedules/schedule_form";
import { useAppStore } from "../../../src/stores/app_store";
import { makeCli, makeSchedule } from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() =>
  resetAppStore({
    clis: [makeCli()],
    perform: vi.fn().mockResolvedValue(true),
  }),
);
it("여러 줄 프롬프트와 실행 시각을 저장한다", async () => {
  const onClose = vi.fn();
  render(<ScheduleForm onClose={onClose} />);
  fireEvent.change(screen.getByLabelText("작업 이름"), {
    target: { value: "예약 테스트" },
  });
  fireEvent.change(screen.getByLabelText("프롬프트"), {
    target: { value: '한글 " $() &\n두 번째 줄' },
  });
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  expect(useAppStore.getState().perform).toHaveBeenCalledWith("save_schedule", {
    id: null,
    input: expect.objectContaining({
      name: "예약 테스트",
      prompt: '한글 " $() &\n두 번째 줄',
      keep_open: false,
      scheduled_at: expect.any(String),
    }),
  });
});
it("예약 수정 시 기존 값과 CLI 경로를 유지한다", async () => {
  const schedule = makeSchedule({
    name: "기존 작업",
    cli_path: "C:/original/codex.exe",
  });
  render(<ScheduleForm schedule={schedule} onClose={() => {}} />);
  expect((screen.getByLabelText("작업 이름") as HTMLInputElement).value).toBe(
    "기존 작업",
  );
  expect(screen.getByText(schedule.cli_path)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "save_schedule",
      {
        id: schedule.id,
        input: expect.objectContaining({ cli_path: schedule.cli_path }),
      },
    ),
  );
});
it("저장 실패 시 폼을 닫지 않는다", async () => {
  useAppStore.setState({ perform: vi.fn().mockResolvedValue(false) });
  const onClose = vi.fn();
  render(<ScheduleForm schedule={makeSchedule()} onClose={onClose} />);
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalled(),
  );
  expect(onClose).not.toHaveBeenCalled();
});
