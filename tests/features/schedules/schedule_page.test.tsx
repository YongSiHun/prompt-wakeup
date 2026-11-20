import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
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
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "예약 취소" }));
  expect(useAppStore.getState().perform).toHaveBeenCalledWith(
    "schedule_action",
    { id: "job-1", action: "cancel" },
  );
});
it("대기 작업이 없으면 새 예약을 안내한다", () => {
  render(<SchedulePage />);
  expect(screen.getByRole("button", { name: "첫 예약 만들기" })).toBeTruthy();
});

it("삭제 대신 마지막 위치에 취소를 표시한다", () => {
  useAppStore.setState({ data: makeSnapshot({ schedules: [makeSchedule()] }) });
  render(<SchedulePage />);
  expect(screen.queryByRole("button", { name: /삭제/ })).toBeNull();
  expect(
    within(screen.getByRole("article"))
      .getAllByRole("button")
      .map((button) => button.textContent),
  ).toEqual(["수정", "지금 실행", "취소"]);
});

it("예약 취소 확인을 거절하면 작업을 유지한다", () => {
  useAppStore.setState({ data: makeSnapshot({ schedules: [makeSchedule()] }) });
  render(<SchedulePage />);
  fireEvent.click(screen.getByRole("button", { name: "취소" }));
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "돌아가기" }));
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
});

it("취소 저장에 실패하면 확인 모달을 유지한다", async () => {
  useAppStore.setState({
    data: makeSnapshot({ schedules: [makeSchedule()] }),
    perform: vi.fn().mockResolvedValue(false),
  });
  render(<SchedulePage />);
  fireEvent.click(screen.getByRole("button", { name: "취소" }));
  fireEvent.click(screen.getByRole("button", { name: "예약 취소" }));
  await waitFor(() =>
    expect(
      (screen.getByRole("button", { name: "예약 취소" }) as HTMLButtonElement)
        .disabled,
    ).toBe(false),
  );
  expect(
    screen.getByRole("dialog", { name: "예약을 취소할까요?" }),
  ).toBeTruthy();
});
