import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ExecutionFile } from "../../../src/features/executions/execution_file";
import { makeExecution } from "../../support/fixtures";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => vi.mocked(invoke).mockReset());
it("전체 경로를 표시하고 선택한 기록의 파일·폴더를 연다", async () => {
  const path = "C:/Users/test/AppData/Local/PromptWakeup/runs/run-1.output";
  vi.mocked(invoke).mockResolvedValueOnce(path).mockResolvedValue(undefined);
  render(<ExecutionFile record={makeExecution()} />);
  await waitFor(() =>
    expect(screen.getByLabelText("첫 작업 출력 파일 경로").textContent).toBe(
      path,
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
it("파일 열기 실패를 표시한다", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce("C:/runs/run-1.output")
    .mockRejectedValue("파일 없음");
  render(<ExecutionFile record={makeExecution()} />);
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "첫 작업 출력 파일 열기",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
  fireEvent.click(
    screen.getByRole("button", { name: "첫 작업 출력 파일 열기" }),
  );
  await waitFor(() => expect(screen.getByText(/파일 없음/)).toBeTruthy());
});
