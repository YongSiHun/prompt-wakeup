import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { CliCatalog } from "../../../src/features/ai_clis/cli_catalog";
import { useAppStore } from "../../../src/stores/app_store";
import { makeCli } from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() => {
  resetAppStore({ refresh: vi.fn().mockResolvedValue(undefined) });
  vi.mocked(invoke).mockReset();
});
it("탐지 결과가 없어도 이름과 경로로 CLI를 등록한다", async () => {
  const cli = makeCli({
    kind: "claude",
    executable_path: "C:/custom/claude.exe",
  });
  vi.mocked(invoke).mockResolvedValue(cli);
  render(<CliCatalog clis={[]} busy={false} onDetect={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "+ 추가" }));
  fireEvent.change(screen.getByLabelText("CLI 이름"), {
    target: { value: cli.kind },
  });
  fireEvent.change(screen.getByLabelText("실행 파일 경로"), {
    target: { value: "  " + cli.executable_path + "  " },
  });
  fireEvent.click(screen.getByRole("button", { name: "저장" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(invoke).toHaveBeenCalledWith("manual_cli", {
    kind: cli.kind,
    path: cli.executable_path,
  });
  expect(useAppStore.getState().clis).toEqual([cli]);
});
it("재탐색 버튼이 탐색을 요청한다", () => {
  const detect = vi.fn();
  render(<CliCatalog clis={[]} busy={false} onDetect={detect} />);
  fireEvent.click(screen.getByRole("button", { name: "CLI 재탐색" }));
  expect(detect).toHaveBeenCalledOnce();
});
