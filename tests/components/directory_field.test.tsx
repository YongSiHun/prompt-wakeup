import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { open } from "@tauri-apps/plugin-dialog";
import { DirectoryField } from "../../src/components/directory_field";
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() => vi.mocked(open).mockReset());
it("선택한 폴더 경로를 전달한다", async () => {
  vi.mocked(open).mockResolvedValue("C:/선택 폴더");
  const onChange = vi.fn();
  render(<DirectoryField label="작업 폴더" value="" onChange={onChange} />);
  fireEvent.click(screen.getByRole("button", { name: "폴더 선택" }));
  await waitFor(() => expect(onChange).toHaveBeenCalledWith("C:/선택 폴더"));
});
it("폴더 선택을 취소하면 기존 경로를 유지한다", async () => {
  vi.mocked(open).mockResolvedValue(null);
  const onChange = vi.fn();
  render(
    <DirectoryField
      label="작업 폴더"
      value="C:/기존 폴더"
      onChange={onChange}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "폴더 선택" }));
  await waitFor(() => expect(open).toHaveBeenCalledOnce());
  expect(onChange).not.toHaveBeenCalled();
});
