import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ExecutionList } from "../../../src/features/executions/execution_list";
import { useAppStore } from "../../../src/stores/app_store";
import {
  makeExecution,
  makeSchedule,
  makeSnapshot,
} from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  resetAppStore({ perform: vi.fn().mockResolvedValue(true) });
  vi.mocked(invoke)
    .mockReset()
    .mockImplementation(async (command, args) => {
      const id = args && "id" in args ? args.id : undefined;
      if (command === "execution_output_path")
        return "C:/runs/" + id + ".output";
      return id === "run-1" ? "첫 작업 stdout\n첫 작업 stderr" : "둘째 결과";
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
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
  const dialog = screen.getByRole("dialog", { name: "삭제하시겠습니까?" });
  fireEvent.click(within(dialog).getByRole("button", { name: "삭제" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "delete_execution",
      { id: "done" },
    ),
  );
});
it("삭제된 예약의 실패 결과도 상태와 오류를 표시한다", async () => {
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
  expect(screen.getByText(/상태: 실패/)).toBeTruthy();
  expect(screen.queryByText(/종료 코드/)).toBeNull();
});
it("기록이 없으면 빈 상태를 표시한다", () => {
  render(<ExecutionList />);
  expect(screen.getByText("아직 실행 이력이 없습니다.")).toBeTruthy();
});

it("예약을 수정하거나 삭제해도 실행 당시 원문을 표시한다", async () => {
  const prompt = "한글 ' \" & <script>\n다음 줄";
  useAppStore.setState({
    data: makeSnapshot({
      schedules: [makeSchedule({ prompt: "수정된 내용" })],
      executions: [
        makeExecution({
          prompt,
          status: "failed",
          ended_at: "2026-01-01T00:01:00Z",
        }),
      ],
    }),
  });
  render(<ExecutionList />);
  expect(screen.getByLabelText("첫 작업 프롬프트").textContent).toBe(prompt);
  useAppStore.setState({
    data: makeSnapshot({ executions: [makeExecution({ prompt })] }),
  });
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 프롬프트").textContent).toBe(prompt),
  );
});

it("이전 이력은 현재 예약의 프롬프트임을 안내한다", () => {
  useAppStore.setState({
    data: makeSnapshot({
      schedules: [makeSchedule({ prompt: "현재 예약 내용" })],
      executions: [makeExecution({ prompt: null })],
    }),
  });
  render(<ExecutionList />);
  expect(screen.getByLabelText("첫 작업 프롬프트").textContent).toBe(
    "현재 예약 내용",
  );
  expect(screen.getByText(/실행 이후 수정되었을 수 있습니다/)).toBeTruthy();
});

it("예약도 없는 이전 이력은 원문이 저장되지 않았음을 안내한다", () => {
  useAppStore.setState({
    data: makeSnapshot({ executions: [makeExecution({ prompt: null })] }),
  });
  render(<ExecutionList />);
  expect(screen.getByText(/당시 프롬프트가 저장되지 않았습니다/)).toBeTruthy();
  expect(screen.queryByLabelText("첫 작업 프롬프트")).toBeNull();
});

it.each([
  {
    status: "completed" as const,
    ended_at: "2026-01-01T00:01:00Z",
    expected: "성공",
  },
  {
    status: "failed" as const,
    ended_at: "2026-01-01T00:01:00Z",
    expected: "실패",
  },
  { status: "launched" as const, ended_at: null, expected: "실행중" },
])(
  "종료 코드를 대신해 상태 $expected 를 표시한다",
  ({ status, ended_at, expected }) => {
    useAppStore.setState({
      data: makeSnapshot({ executions: [makeExecution({ status, ended_at })] }),
    });
    render(<ExecutionList />);
    expect(screen.getByText(new RegExp("상태: " + expected))).toBeTruthy();
    const terminal = screen.getByRole("article", { name: "첫 작업 터미널" });
    const title = within(terminal).getByRole("heading", { name: "첫 작업" });
    expect(title.parentElement?.querySelector(".badge")).toBeTruthy();
  },
);

it("오래된 이력의 출력을 한꺼번에 불러오지 않고 더 보기로 표시한다", () => {
  const executions = Array.from({ length: 21 }, (_, index) =>
    makeExecution({
      id: "run-" + index,
      name: "작업 " + index,
      status: "completed",
      ended_at: "2026-01-01T00:01:00Z",
    }),
  );
  useAppStore.setState({ data: makeSnapshot({ executions }) });
  render(<ExecutionList />);
  expect(screen.getAllByRole("article")).toHaveLength(20);
  expect(screen.queryByLabelText("작업 0 출력")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "이전 이력 더 보기" }));
  expect(screen.getAllByRole("article")).toHaveLength(21);
  expect(screen.getByLabelText("작업 0 출력")).toBeTruthy();
});
