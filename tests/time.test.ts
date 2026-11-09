import { describe, it, expect, vi, afterEach } from "vitest";
import { localInput, utcFromLocal, remaining } from "../src/lib/time";
describe("예약 시각", () => {
  afterEach(() => vi.useRealTimers());
  it("초 단위 로컬 입력을 UTC로 변환한다", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-01"));
    const d = new Date("2027-03-01T01:02:03Z");
    expect(utcFromLocal(localInput(d))).toBe(d.toISOString());
  });
  it("지난 예약을 거부한다", () =>
    expect(() => utcFromLocal("2000-01-01T00:00:00")).toThrow());
  it("잘못된 시각을 거부한다", () =>
    expect(() => utcFromLocal("not-a-date")).toThrow());
  it("존재하지 않는 달력 날짜를 거부한다", () =>
    expect(() => utcFromLocal("2030-02-30T12:00:00")).toThrow());
  it("남은 시간과 분을 표시한다", () =>
    expect(
      remaining("2030-01-01T01:02:03Z", Date.parse("2030-01-01T00:00:00Z")),
    ).toBe("1시간 2분 3초"));
  it("지난 시각의 카운트다운을 표시한다", () =>
    expect(
      remaining("2030-01-01T00:00:00Z", Date.parse("2030-01-01T00:00:01Z")),
    ).toBe("실행 시각 도달"));
  it("남은 초를 표시한다", () =>
    expect(
      remaining("2027-01-01T00:00:59Z", Date.parse("2027-01-01T00:00:00Z")),
    ).toBe("59초"));
});
