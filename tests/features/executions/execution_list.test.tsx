import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ExecutionList } from "../../../src/features/executions/execution_list";
import { useAppStore } from "../../../src/stores/app_store";
import { makeExecution, makeSnapshot } from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  resetAppStore({ perform: vi.fn().mockResolvedValue(true) });
  vi.mocked(invoke)
    .mockReset()
    .mockImplementation(async (command, args) => {
      if (command === "execution_output_path")
        return "C:/runs/" + args?.id + ".output";
      return args?.id === "run-1"
        ? "첫 작업 stdout\n첫 작업 stderr"
        : "둘째 결과";
    });
});
it("작업별 출력과 오류를 분리해서 표시한다", async () => {
  useAppStore.setState({
    data: makeSnapshot({
      executions: [
        makeExecution(),
        makeExecution({ id: "run-2", name: "둘째 작업" }),
      ],
    }),
  });
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toContain(
      "첫 작업 stderr",
    ),
  );
  expect(screen.getByLabelText("둘째 작업 출력").textContent).toBe("둘째 결과");
});
it("완료된 결과만 삭제할 수 있다", async () => {
  useAppStore.setState({
    data: makeSnapshot({
      executions: [
        makeExecution(),
        makeExecution({
          id: "done",
          name: "완료 작업",
          status: "completed",
          ended_at: "2026-01-01T00:01:00Z",
        }),
      ],
    }),
  });
  render(<ExecutionList />);
  expect(
    (
      screen.getByRole("button", {
        name: "첫 작업 실행 결과 삭제",
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(true);
  fireEvent.click(
    screen.getByRole("button", { name: "완료 작업 실행 결과 삭제" }),
  );
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "delete_execution",
      { id: "done" },
    ),
  );
});
it("삭제된 예약의 실패 결과도 종료 코드와 오류를 표시한다", async () => {
  useAppStore.setState({
    data: makeSnapshot({
      executions: [
        makeExecution({
          name: "삭제된 작업",
          schedule_id: "deleted",
          status: "failed",
          ended_at: "2026-01-01T00:01:00Z",
          exit_code: 7,
          error: "권한 거부",
        }),
      ],
    }),
  });
  render(<ExecutionList />);
  await waitFor(() => expect(screen.getByText("권한 거부")).toBeTruthy());
  expect(screen.getByText(/종료 코드: 7/)).toBeTruthy();
});
it("기록이 없으면 빈 상태를 표시한다", () => {
  render(<ExecutionList />);
  expect(screen.getByText("아직 실행 이력이 없습니다.")).toBeTruthy();
});
