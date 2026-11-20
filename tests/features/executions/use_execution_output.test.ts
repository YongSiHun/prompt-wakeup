import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useExecutionOutput } from "../../../src/features/executions/use_execution_output";
import { makeExecution } from "../../support/fixtures";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => vi.mocked(invoke).mockReset());
it("실행 중 출력을 갱신하고 완료 시 마지막 출력을 읽고 갱신을 멈춘다", async () => {
  vi.useFakeTimers();
  vi.mocked(invoke)
    .mockResolvedValueOnce("진행 중")
    .mockResolvedValueOnce("추가 출력")
    .mockResolvedValue("최종 결과");
  const { result, rerender, unmount } = renderHook(useExecutionOutput, {
    initialProps: makeExecution(),
  });
  await act(async () => {
    await Promise.resolve();
  });
  expect(result.current.output).toBe("진행 중");
  await act(async () => {
    await vi.advanceTimersByTimeAsync(1000);
  });
  expect(result.current.output).toBe("추가 출력");
  rerender(
    makeExecution({ status: "completed", ended_at: "2026-01-01T00:01:00Z" }),
  );
  await act(async () => {
    await Promise.resolve();
  });
  expect(result.current.output).toBe("최종 결과");
  expect(vi.getTimerCount()).toBe(0);
  unmount();
});
it("완료된 기록은 화면 재진입 시 저장된 전체 출력을 읽는다", async () => {
  const text = "한글 결과\n".repeat(20000);
  vi.mocked(invoke).mockResolvedValue(text);
  const record = makeExecution({
    status: "completed",
    ended_at: "2026-01-01T00:01:00Z",
  });
  const first = renderHook(() => useExecutionOutput(record));
  await waitFor(() => expect(first.result.current.output).toBe(text));
  first.unmount();
  const second = renderHook(() => useExecutionOutput(record));
  await waitFor(() => expect(second.result.current.output).toBe(text));
  expect(invoke).toHaveBeenCalledTimes(2);
});
it("출력 조회 오류를 반환한다", async () => {
  vi.mocked(invoke).mockRejectedValue("읽기 실패");
  const { result } = renderHook(() => useExecutionOutput(makeExecution()));
  await waitFor(() => expect(result.current.error).toBe("읽기 실패"));
});
it("화면을 떠나면 출력 갱신 타이머를 정리한다", async () => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockResolvedValue("진행 중");
  const { unmount } = renderHook(() => useExecutionOutput(makeExecution()));
  await act(async () => {
    await Promise.resolve();
  });
  expect(vi.getTimerCount()).toBe(1);
  unmount();
  expect(vi.getTimerCount()).toBe(0);
});

it("다른 기록으로 전환하면 이전 출력을 지우고 늦은 응답을 무시한다", async () => {
  const { deferred } = await import("../../support/deferred");
  const oldRequest = deferred<string>();
  const newRequest = deferred<string>();
  vi.mocked(invoke)
    .mockReturnValueOnce(oldRequest.promise)
    .mockReturnValueOnce(newRequest.promise);
  const { result, rerender } = renderHook(useExecutionOutput, {
    initialProps: makeExecution(),
  });
  rerender(makeExecution({ id: "run-2" }));
  expect(result.current.output).toBe("");
  await act(async () => {
    newRequest.resolve("새 작업 결과");
  });
  expect(result.current.output).toBe("새 작업 결과");
  await act(async () => {
    oldRequest.resolve("이전 작업 결과");
  });
  expect(result.current.output).toBe("새 작업 결과");
});
