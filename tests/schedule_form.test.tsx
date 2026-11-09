import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { SettingsPanel } from "../src/features/settings/settings_panel";
import {
  act,
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ScheduleForm } from "../src/features/schedules/schedule_form";
import { ExecutionList } from "../src/features/executions/execution_list";
import { useAppStore } from "../src/stores/app_store";
import type { Snapshot, TerminalInfo } from "../src/types";
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const terminal: TerminalInfo = {
  id: "cmd",
  name: "Command Prompt",
  kind: "cmd",
  executable_path: "C:/Windows/System32/cmd.exe",
  detected_automatically: true,
  installed: true,
  supported: true,
  validation_error: null,
};
const data: Snapshot = {
  active: true,
  warnings: [],
  schedules: [],
  executions: [],
  settings: {
    default_cli: "codex",
    default_terminal: "cmd",
    default_directory: "C:/작업 폴더",
    theme: "system",
    keep_open: true,
    notifications: false,
    retention_days: 90,
    custom_cli_paths: {},
    custom_terminals: [],
  },
};
describe("예약 UI", () => {
  beforeEach(() =>
    useAppStore.setState({
      data: structuredClone(data),
      terminals: [terminal],
      clis: [
        {
          kind: "codex",
          executable_path: "C:/tools/codex.exe",
          available: true,
          version: "1",
          validation_error: null,
        },
      ],
      busy: false,
      error: null,
    }),
  );
  afterEach(cleanup);
  it("한글 여러 줄 프롬프트와 초 단위 시각을 백엔드에 전달한다", async () => {
    const perform = vi.fn().mockResolvedValue(true);
    useAppStore.setState({ perform });
    const close = vi.fn();
    render(<ScheduleForm onClose={close} />);
    fireEvent.change(screen.getByLabelText("작업 이름"), {
      target: { value: "예약 테스트" },
    });
    fireEvent.change(screen.getByLabelText("프롬프트"), {
      target: { value: '한글 \" $() &\n두 번째 줄' },
    });
    fireEvent.click(screen.getByText("예약 저장"));
    await waitFor(() => expect(perform).toHaveBeenCalled());
    expect(perform.mock.calls[0][0]).toBe("save_schedule");
    expect(perform.mock.calls[0][1].input.prompt).toContain("두 번째 줄");
    expect(close).toHaveBeenCalled();
  });
  it("기존 예약 값을 수정 폼에 표시한다", () => {
    render(
      <ScheduleForm
        schedule={{
          id: "job",
          name: "기존 작업",
          cli: "codex",
          cli_path: "C:/tools/codex.exe",
          terminal,
          working_directory: "C:/작업",
          prompt: "기존 프롬프트",
          scheduled_at: "2027-01-01T00:00:00Z",
          local_time: "2027-01-01T09:00:00",
          timezone: "Asia/Seoul",
          keep_open: false,
          status: "pending",
        }}
        onClose={() => {}}
      />,
    );
    expect((screen.getByLabelText("작업 이름") as HTMLInputElement).value).toBe(
      "기존 작업",
    );
  });
  it("종료 코드를 포함한 실행 이력을 표시한다", () => {
    useAppStore.setState({
      data: {
        ...data,
        executions: [
          {
            id: "run",
            schedule_id: "deleted",
            name: "삭제된 작업",
            cli: "codex",
            status: "failed",
            started_at: "2026-01-01T00:00:00Z",
            ended_at: "2026-01-01T00:01:00Z",
            exit_code: 7,
            error: "권한 거부",
          },
        ],
      },
    });
    render(<ExecutionList />);
    expect(screen.getByText("삭제된 작업")).toBeTruthy();
    expect(screen.getByText("권한 거부")).toBeTruthy();
  });
});

const customTerminal: TerminalInfo = {
  ...terminal,
  id: "cmd:C:/custom/cmd.exe",
  executable_path: "C:/custom/cmd.exe",
  detected_automatically: false,
};
describe("실행 파일 선택", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAppStore.setState({
      data: structuredClone(data),
      terminals: [terminal],
      clis: [],
      busy: false,
      error: null,
      refresh: vi.fn().mockResolvedValue(undefined),
      perform: vi.fn().mockResolvedValue(true),
    });
  });
  afterEach(cleanup);
  it("예약 화면에서 선택한 터미널과 CLI 경로로 저장한다", async () => {
    vi.mocked(open)
      .mockResolvedValueOnce("C:/custom/cmd.exe")
      .mockResolvedValueOnce("C:/custom/codex.cmd");
    vi.mocked(invoke)
      .mockResolvedValueOnce(customTerminal)
      .mockResolvedValueOnce({
        kind: "codex",
        executable_path: "C:/custom/codex.cmd",
        available: true,
        version: "1",
        validation_error: null,
      });
    render(<ScheduleForm onClose={() => {}} />);
    fireEvent.click(screen.getByText("터미널 실행 파일 선택"));
    await waitFor(() =>
      expect(
        (screen.getByLabelText("외부 터미널") as HTMLSelectElement).value,
      ).toBe(customTerminal.id),
    );
    fireEvent.click(screen.getByText("AI CLI 실행 파일 선택"));
    await waitFor(() =>
      expect(screen.getByText("C:/custom/codex.cmd")).toBeTruthy(),
    );
    fireEvent.change(screen.getByLabelText("작업 이름"), {
      target: { value: "직접 선택" },
    });
    fireEvent.change(screen.getByLabelText("프롬프트"), {
      target: { value: "실행" },
    });
    fireEvent.click(screen.getByText("예약 저장"));
    await waitFor(() =>
      expect(useAppStore.getState().perform).toHaveBeenCalledWith(
        "save_schedule",
        {
          id: null,
          input: expect.objectContaining({
            terminal: customTerminal,
            cli_path: "C:/custom/codex.cmd",
          }),
        },
      ),
    );
    expect(invoke).toHaveBeenCalledWith("manual_terminal", {
      path: "C:/custom/cmd.exe",
    });
    expect(invoke).toHaveBeenCalledWith("manual_cli", {
      kind: "codex",
      path: "C:/custom/codex.cmd",
    });
  });
  it("파일 선택 취소 시 기존 터미널을 유지한다", async () => {
    vi.mocked(open).mockResolvedValueOnce(null);
    render(<ScheduleForm onClose={() => {}} />);
    fireEvent.click(screen.getByText("터미널 실행 파일 선택"));
    await waitFor(() =>
      expect(
        (screen.getByText("터미널 실행 파일 선택") as HTMLButtonElement)
          .disabled,
      ).toBe(false),
    );
    expect(invoke).not.toHaveBeenCalled();
    expect(
      (screen.getByLabelText("외부 터미널") as HTMLSelectElement).value,
    ).toBe(terminal.id);
  });
  it("파일 검증 실패 시 오류를 표시하고 선택을 유지한다", async () => {
    vi.mocked(open).mockResolvedValueOnce("C:/wrong.exe");
    vi.mocked(invoke).mockRejectedValueOnce(
      "지원되는 터미널을 다시 선택하세요.",
    );
    render(<ScheduleForm onClose={() => {}} />);
    fireEvent.click(screen.getByText("터미널 실행 파일 선택"));
    await waitFor(() =>
      expect(useAppStore.getState().error).toBe(
        "지원되는 터미널을 다시 선택하세요.",
      ),
    );
    expect(useAppStore.getState().terminals).toEqual([terminal]);
  });
  it("탐지된 CLI가 없어도 설정에서 파일을 지정할 수 있다", async () => {
    vi.mocked(open).mockResolvedValueOnce("C:/custom/claude.exe");
    vi.mocked(invoke).mockResolvedValueOnce({
      kind: "claude",
      executable_path: "C:/custom/claude.exe",
      available: true,
      version: "1",
      validation_error: null,
    });
    render(<SettingsPanel />);
    fireEvent.click(screen.getAllByText("실행 파일 지정")[1]);
    await waitFor(() =>
      expect(
        (screen.getByLabelText("기본 AI CLI") as HTMLSelectElement).value,
      ).toBe("claude"),
    );
    expect(screen.getByText("C:/custom/claude.exe")).toBeTruthy();
  });
  it("같은 종류의 터미널도 선택한 파일을 기본값으로 저장한다", async () => {
    vi.mocked(open).mockResolvedValueOnce(customTerminal.executable_path);
    vi.mocked(invoke).mockResolvedValueOnce(customTerminal);
    render(<SettingsPanel />);
    fireEvent.click(screen.getByText("터미널 수동 선택"));
    await waitFor(() =>
      expect(
        (screen.getByLabelText("기본 터미널") as HTMLSelectElement).value,
      ).toBe(customTerminal.id),
    );
    fireEvent.click(screen.getByText("설정 저장"));
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "save_settings",
      {
        settings: expect.objectContaining({
          default_terminal: customTerminal.id,
        }),
      },
    );
  });
  it("새 예약에서 기본 터미널의 파일 경로를 구분한다", () => {
    useAppStore.setState({
      terminals: [terminal, customTerminal],
      data: {
        ...data,
        settings: { ...data.settings, default_terminal: customTerminal.id },
      },
    });
    render(<ScheduleForm onClose={() => {}} />);
    expect(
      (screen.getByLabelText("외부 터미널") as HTMLSelectElement).value,
    ).toBe(customTerminal.id);
  });
});

describe("예약 경로 보존", () => {
  beforeEach(() => {
    useAppStore.setState({
      data: structuredClone(data),
      terminals: [terminal],
      busy: false,
      error: null,
      clis: [
        {
          kind: "codex",
          executable_path: "C:/new/codex.exe",
          available: true,
          version: "1",
          validation_error: null,
        },
      ],
      perform: vi.fn().mockResolvedValue(true),
    });
  });
  afterEach(cleanup);
  it("기존 예약을 수정할 때 저장된 CLI 경로를 유지한다", async () => {
    render(
      <ScheduleForm
        schedule={{
          id: "existing",
          name: "기존 작업",
          cli: "codex",
          cli_path: "C:/original/codex.exe",
          terminal,
          working_directory: "C:/작업",
          prompt: "작업",
          scheduled_at: "2037-01-01T00:00:00Z",
          local_time: "2037-01-01T09:00:00",
          timezone: "Asia/Seoul",
          keep_open: false,
          status: "pending",
        }}
        onClose={() => {}}
      />,
    );
    expect(screen.getByText("C:/original/codex.exe")).toBeTruthy();
    fireEvent.click(screen.getByText("예약 저장"));
    await waitFor(() =>
      expect(useAppStore.getState().perform).toHaveBeenCalledWith(
        "save_schedule",
        {
          id: "existing",
          input: expect.objectContaining({ cli_path: "C:/original/codex.exe" }),
        },
      ),
    );
  });
  it("폼을 연 뒤 탐지된 기본 터미널도 선택한다", async () => {
    useAppStore.setState({ terminals: [] });
    render(<ScheduleForm onClose={() => {}} />);
    act(() => useAppStore.getState().addTerminal(terminal));
    await waitFor(() =>
      expect(
        (screen.getByLabelText("외부 터미널") as HTMLSelectElement).value,
      ).toBe(terminal.id),
    );
  });
});
