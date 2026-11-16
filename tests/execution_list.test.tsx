import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ExecutionList } from "../src/features/executions/execution_list";
import { useAppStore } from "../src/stores/app_store";
import type { ExecutionRecord, Snapshot } from "../src/types";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
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
  vi.mocked(invoke).mockImplementation(async (_command, args) =>
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
  vi.mocked(invoke)
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
  const count = vi.mocked(invoke).mock.calls.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(3000);
  });
  expect(vi.mocked(invoke).mock.calls).toHaveLength(count);
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
  vi.mocked(invoke).mockResolvedValue(text);
  const view = render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toBe(text),
  );
  view.unmount();
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력").textContent).toBe(text),
  );
  expect(invoke).toHaveBeenCalledTimes(2);
});

it("출력 조회 오류를 표시한다", async () => {
  vi.mocked(invoke).mockRejectedValue("읽기 실패");
  render(<ExecutionList />);
  await waitFor(() =>
    expect(screen.getByRole("alert").textContent).toContain("읽기 실패"),
  );
});
