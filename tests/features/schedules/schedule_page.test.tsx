import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { SchedulePage } from "../../../src/features/schedules/schedule_page";
import { useAppStore } from "../../../src/stores/app_store";
import { makeSchedule, makeSnapshot } from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() => resetAppStore({ perform: vi.fn().mockResolvedValue(true) }));
it("대기 작업만 표시하고 실행 결과와 실행 환경은 표시하지 않는다", () => {
  const statuses = [
    "completed",
    "failed",
    "canceled",
    "missed",
    "launching",
    "launched",
  ] as const;
  useAppStore.setState({
    data: makeSnapshot({
      schedules: [
        makeSchedule(),
        ...statuses.map((status) =>
          makeSchedule({ id: status, name: "숨김 " + status, status }),
        ),
      ],
    }),
  });
  render(<SchedulePage />);
  expect(screen.getByText("예약 테스트")).toBeTruthy();
  expect(screen.queryByText(/숨김/)).toBeNull();
  expect(screen.queryByText("실행 환경")).toBeNull();
  expect(screen.queryByRole("heading", { name: "실행이력" })).toBeNull();
});
it("취소 액션을 선택한 예약에 전달한다", () => {
  useAppStore.setState({ data: makeSnapshot({ schedules: [makeSchedule()] }) });
  render(<SchedulePage />);
  fireEvent.click(screen.getByRole("button", { name: "취소" }));
  expect(useAppStore.getState().perform).toHaveBeenCalledWith(
    "schedule_action",
    { id: "job-1", action: "cancel" },
  );
});
it("대기 작업이 없으면 새 예약을 안내한다", () => {
  render(<SchedulePage />);
  expect(screen.getByRole("button", { name: "첫 예약 만들기" })).toBeTruthy();
});

it("예약 삭제는 확인 후에만 실행한다", () => {
  useAppStore.setState({ data: makeSnapshot({ schedules: [makeSchedule()] }) });
  render(<SchedulePage />);
  fireEvent.click(
    screen.getByRole("button", { name: "예약 테스트 예약 삭제" }),
  );
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
  fireEvent.click(
    within(screen.getByRole("dialog", { name: "삭제하시겠습니까?" })).getByRole(
      "button",
      { name: "삭제" },
    ),
  );
  expect(useAppStore.getState().perform).toHaveBeenCalledWith(
    "schedule_action",
    { id: "job-1", action: "delete" },
  );
});
