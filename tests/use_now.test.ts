import { renderHook, cleanup } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useNow } from "../src/hooks/use_now";
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});
it("여러 카운트다운이 하나의 타이머를 공유하고 마지막 구독 해제 시 정리한다", () => {
  vi.useFakeTimers();
  const first = renderHook(useNow);
  const second = renderHook(useNow);
  expect(vi.getTimerCount()).toBe(1);
  first.unmount();
  expect(vi.getTimerCount()).toBe(1);
  second.unmount();
  expect(vi.getTimerCount()).toBe(0);
});
