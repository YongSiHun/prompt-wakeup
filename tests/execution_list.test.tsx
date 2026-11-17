import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ExecutionList } from "../src/features/executions/execution_list";
import { useAppStore } from "../src/stores/app_store";
import type { ExecutionRecord, Snapshot } from "../src/types";
const { readOutput } = vi.hoisted(() => ({ readOutput: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((command, args) => {
    if (command === "execution_output_path")
      return Promise.resolve(
        "C:/Users/test/AppData/Local/PromptWakeup/runs/" + args.id + ".output",
      );
    if (command === "open_execution_output") return Promise.resolve();
    return readOutput(command, args);
  }),
}));
const record: ExecutionRecord = {
  id: "run-1",
  schedule_id: "job",
  name: "첫 작업",
  cli: "codex",
  started_at: "2026-01-01T00:00:00Z",
  ended_at: null,
  status: "launched",
  exit_code: null,
  error: null,
};
function setRecords(executions: ExecutionRecord[]) {
  useAppStore.setState({ data: { executions } as Snapshot });
}
beforeEach(() => {
  vi.clearAllMocks();
  setRecords([record]);
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

it("작업별 출력과 오류를 분리해 표시한다", async () => {
  setRecords([record, { ...record, id: "run-2", name: "둘째 작업" }]);
  readOutput.mockImplementation(async (_command, args) =>
    (args as { id: string }).id === "run-1"
      ? "첫 작업 stdout\n첫 작업 stderr"
      : "둘째 결과",
  );
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toContain(
      "첫 작업 stderr",
    ),
  );
  expect(screen.getByLabelText("둘째 작업 출력").textContent).toBe("둘째 결과");
});

it("실행 중 갱신하고 완료 시 마지막 출력을 읽는다", async () => {
  vi.useFakeTimers();
  readOutput
    .mockResolvedValueOnce("진행 중")
    .mockResolvedValueOnce("추가 출력")
    .mockResolvedValue("최종 결과");
  render(<ExecutionList />);
  await act(async () => {
    await Promise.resolve();
  });
  expect(screen.getByLabelText("첫 작업 출력").textContent).toBe("진행 중");
  await act(async () => {
    await vi.advanceTimersByTimeAsync(1000);
  });
  expect(screen.getByLabelText("첫 작업 출력").textContent).toBe("추가 출력");
  await act(async () => {
    setRecords([
      {
        ...record,
        status: "completed",
        ended_at: "2026-01-01T00:01:00Z",
        exit_code: 0,
      },
    ]);
  });
  expect(screen.getByLabelText("첫 작업 출력").textContent).toBe("최종 결과");
  const count = readOutput.mock.calls.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  expect(readOutput.mock.calls).toHaveLength(count);
});

it("화면에 돌아오면 저장된 전체 출력을 다시 읽는다", async () => {
  const text = "한글 결과\n".repeat(20000);
  setRecords([
    {
      ...record,
      status: "completed",
      ended_at: "2026-01-01T00:01:00Z",
      exit_code: 0,
    },
  ]);
  readOutput.mockResolvedValue(text);
  const view = render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toBe(text),
  );
  view.unmount();
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toBe(text),
  );
  expect(readOutput).toHaveBeenCalledTimes(2);
});

it("출력 조회 오류를 표시한다", async () => {
  readOutput.mockRejectedValue("읽기 실패");
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByRole("alert").textContent).toContain("읽기 실패"),
  );
});

it("완료된 실행 결과만 삭제할 수 있다", async () => {
  const perform = vi.fn().mockResolvedValue(true);
  useAppStore.setState({ perform, busy: false });
  setRecords([
    record,
    {
      ...record,
      id: "done",
      name: "완료 작업",
      status: "completed",
      ended_at: "2026-01-01T00:01:00Z",
    },
  ]);
  readOutput.mockResolvedValue("출력");
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
    expect(perform).toHaveBeenCalledWith("delete_execution", { id: "done" }),
  );
});

it("실제 출력 파일 경로를 표시하고 파일과 폴더를 연다", async () => {
  readOutput.mockResolvedValue("작업 출력");
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력 파일 경로").textContent).toBe(
      "C:/Users/test/AppData/Local/PromptWakeup/runs/run-1.output",
    ),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "첫 작업 출력 파일 열기" }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("open_execution_output", {
      id: "run-1",
      folder: false,
    }),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "첫 작업 저장 폴더 열기" }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("open_execution_output", {
      id: "run-1",
      folder: true,
    }),
  );
});
