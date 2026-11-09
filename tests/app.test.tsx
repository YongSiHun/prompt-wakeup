import {
  render,
  screen,
  fireEvent,
  cleanup,
  waitFor,
} from "@testing-library/react";
import { it, expect, vi, afterEach } from "vitest";
import App from "../src/App";
import { useAppStore } from "../src/stores/app_store";
vi.mock("../src/hooks/use_backend", () => ({ useBackend: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(cleanup);
it("예약 목록과 취소 액션을 표시한다", async () => {
  const perform = vi.fn().mockResolvedValue(true);
  useAppStore.setState({
    perform,
    data: {
      active: true,
      warnings: [],
      executions: [],
      settings: {
        default_cli: "codex",
        default_terminal: "cmd",
        default_directory: "C:/작업",
        theme: "light",
        keep_open: true,
        notifications: false,
        retention_days: 90,
        custom_cli_paths: {},
        custom_terminals: [],
      },
      schedules: [
        {
          id: "job-1",
          name: "예약 목록 테스트",
          cli: "codex",
          cli_path: "C:/codex.exe",
          terminal: {
            id: "cmd",
            name: "Command Prompt",
            kind: "cmd",
            executable_path: "C:/Windows/System32/cmd.exe",
            installed: true,
            supported: true,
            detected_automatically: true,
            validation_error: null,
          },
          working_directory: "C:/작업",
          prompt: "test",
          scheduled_at: "2027-01-01T00:00:00Z",
          timezone: "Asia/Seoul",
          local_time: "2027-01-01T09:00:00",
          keep_open: true,
          status: "pending",
        },
      ],
    },
  });
  render(<App />);
  expect(screen.getByText("예약 목록 테스트")).toBeTruthy();
  fireEvent.click(screen.getByText("취소"));
  await waitFor(() =>
    expect(perform).toHaveBeenCalledWith("schedule_action", {
      id: "job-1",
      action: "cancel",
    }),
  );
});
