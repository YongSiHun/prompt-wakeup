import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ScheduleForm } from "../../../src/features/schedules/schedule_form";
import { useAppStore } from "../../../src/stores/app_store";
import { makeCli, makeSchedule } from "../../support/fixtures";
import { resetAppStore } from "../../support/app_store";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() =>
  resetAppStore({
    clis: [makeCli()],
    perform: vi.fn().mockResolvedValue(true),
  }),
);
it("여러 줄 프롬프트와 실행 시각을 저장한다", async () => {
  const onClose = vi.fn();
  render(<ScheduleForm onClose={onClose} />);
  fireEvent.change(screen.getByLabelText("작업 이름"), {
    target: { value: "예약 테스트" },
  });
  fireEvent.change(screen.getByLabelText("프롬프트"), {
    target: { value: '한글 " $() &\n두 번째 줄' },
  });
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  expect(useAppStore.getState().perform).toHaveBeenCalledWith("save_schedule", {
    id: null,
    input: expect.objectContaining({
      name: "예약 테스트",
      prompt: '한글 " $() &\n두 번째 줄',
      keep_open: false,
      scheduled_at: expect.any(String),
    }),
  });
});
it("예약 수정 시 기존 값과 CLI 경로를 유지한다", async () => {
  const schedule = makeSchedule({
    name: "기존 작업",
    cli_path: "C:/original/codex.exe",
  });
  render(<ScheduleForm schedule={schedule} onClose={() => {}} />);
  expect((screen.getByLabelText("작업 이름") as HTMLInputElement).value).toBe(
    "기존 작업",
  );
  expect(
    screen.getByRole("option", { name: "codex · " + schedule.cli_path }),
  ).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "save_schedule",
      {
        id: schedule.id,
        input: expect.objectContaining({ cli_path: schedule.cli_path }),
      },
    ),
  );
});
it("저장 실패 시 폼을 닫지 않는다", async () => {
  useAppStore.setState({ perform: vi.fn().mockResolvedValue(false) });
  const onClose = vi.fn();
  render(<ScheduleForm schedule={makeSchedule()} onClose={onClose} />);
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalled(),
  );
  expect(onClose).not.toHaveBeenCalled();
});

it("CLI 기본 선택 없이 제목을 미리 채우고 실제 사용 가능한 CLI만 표시한다", () => {
  const data = useAppStore.getState().data!;
  useAppStore.setState({
    data: { ...data, settings: { ...data.settings, default_cli: null } },
    clis: [
      makeCli(),
      makeCli({ kind: "claude", available: false, executable_path: "" }),
    ],
  });
  render(<ScheduleForm onClose={() => {}} />);
  expect((screen.getByLabelText("AI CLI") as HTMLSelectElement).value).toBe("");
  expect((screen.getByLabelText("작업 이름") as HTMLInputElement).value).toBe(
    "새 예약 작업",
  );
  expect(screen.getByRole("option", { name: /codex · C:/ })).toBeTruthy();
  expect(screen.queryByRole("option", { name: /claude/ })).toBeNull();
  expect(screen.queryByRole("heading")).toBeNull();
  expect(screen.queryByRole("button", { name: "닫기" })).toBeNull();
});
it("시분초는 숫자 입력으로 저장하고 잘못된 범위를 거부한다", async () => {
  render(<ScheduleForm schedule={makeSchedule()} onClose={() => {}} />);
  const hour = screen.getByLabelText("시") as HTMLInputElement;
  const minute = screen.getByLabelText("분") as HTMLInputElement;
  expect(hour.type).toBe("text");
  expect(hour.inputMode).toBe("numeric");
  fireEvent.change(hour, { target: { value: "24" } });
  expect(hour.checkValidity()).toBe(false);
  fireEvent.change(hour, { target: { value: "1" } });
  fireEvent.change(minute, { target: { value: "60" } });
  expect(minute.checkValidity()).toBe(false);
  fireEvent.change(minute, { target: { value: "2" } });
  fireEvent.change(screen.getByLabelText("초"), { target: { value: "3" } });
  fireEvent.click(screen.getByRole("button", { name: "예약 저장" }));
  await waitFor(() =>
    expect(useAppStore.getState().perform).toHaveBeenCalledWith(
      "save_schedule",
      expect.objectContaining({
        input: expect.objectContaining({
          local_time: "2037-01-01T01:02:03",
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        }),
      }),
    ),
  );
});

it("감지된 CLI가 없어도 예약 폼에서 CLI를 등록하고 경로를 선택한다", async () => {
  const cli = makeCli({
    kind: "claude",
    executable_path: "C:/custom/claude.exe",
  });
  const data = useAppStore.getState().data!;
  useAppStore.setState({
    data: { ...data, settings: { ...data.settings, default_cli: null } },
    clis: [],
    refresh: vi.fn().mockResolvedValue(undefined),
  });
  vi.mocked(invoke).mockResolvedValue(cli);
  render(<ScheduleForm onClose={() => {}} />);
  fireEvent.click(
    screen.getByRole("button", { name: "AI CLI 실행 파일 선택" }),
  );
  const editor = screen.getByRole("dialog", { name: "CLI 추가" });
  expect(
    (within(editor).getByLabelText("CLI 이름") as HTMLSelectElement).value,
  ).toBe("");
  fireEvent.change(within(editor).getByLabelText("CLI 이름"), {
    target: { value: cli.kind },
  });
  fireEvent.change(within(editor).getByLabelText("실행 파일 경로"), {
    target: { value: cli.executable_path },
  });
  fireEvent.click(within(editor).getByRole("button", { name: "저장" }));
  await waitFor(() =>
    expect(screen.queryByRole("dialog", { name: "CLI 추가" })).toBeNull(),
  );
  expect((screen.getByLabelText("AI CLI") as HTMLSelectElement).value).toBe(
    cli.kind,
  );
  expect(
    screen.getByRole("option", {
      name: cli.kind + " · " + cli.executable_path,
    }),
  ).toBeTruthy();
  expect(useAppStore.getState().perform).not.toHaveBeenCalled();
});
