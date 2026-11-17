import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import App from "../src/App";
import { resetAppStore } from "./support/app_store";
vi.mock("../src/hooks/use_backend", () => ({ useBackend: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
beforeEach(() => resetAppStore());
it("예약·실행 결과·설정 화면을 전환한다", () => {
  render(<App />);
  expect(
    screen.getByRole("heading", { name: "다음 작업을 준비하세요" }),
  ).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "터미널 / 실행 결과" }));
  expect(
    screen.getByRole("heading", { name: "터미널과 실행 결과" }),
  ).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "설정" }));
  expect(screen.getByRole("heading", { name: "CLI 탐지" })).toBeTruthy();
});
