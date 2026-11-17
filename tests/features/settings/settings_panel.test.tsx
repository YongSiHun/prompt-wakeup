import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { SettingsPanel } from "../../../src/features/settings/settings_panel";
import { useAppStore } from "../../../src/stores/app_store";
import { resetAppStore } from "../../support/app_store";
import { makeCli, makeSettings, makeSnapshot } from "../../support/fixtures";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() => resetAppStore({ perform: vi.fn().mockResolvedValue(true) }));
it("미감지 기본 CLI를 비우고 실제 CLI만 선택하게 한다", () => {
  useAppStore.setState({
    data: makeSnapshot({ settings: makeSettings({ default_cli: "claude" }) }),
    clis: [
      makeCli(),
      makeCli({ kind: "claude", available: false, executable_path: "" }),
    ],
  });
  render(<SettingsPanel />);
  const select = screen.getByLabelText("기본 AI CLI") as HTMLSelectElement;
  expect(select.value).toBe("");
  expect(screen.queryByRole("option", { name: /claude/ })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "설정 저장" }));
  expect(useAppStore.getState().perform).toHaveBeenLastCalledWith(
    "save_settings",
    { settings: expect.objectContaining({ default_cli: null }) },
  );
  fireEvent.change(select, { target: { value: "codex" } });
  fireEvent.click(screen.getByRole("button", { name: "설정 저장" }));
  expect(useAppStore.getState().perform).toHaveBeenLastCalledWith(
    "save_settings",
    { settings: expect.objectContaining({ default_cli: "codex" }) },
  );
});
it("탐지된 CLI가 없으면 빈 선택만 표시한다", () => {
  render(<SettingsPanel />);
  expect(
    (screen.getByLabelText("기본 AI CLI") as HTMLSelectElement).options.length,
  ).toBe(1);
});
