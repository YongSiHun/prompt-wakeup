function localDate(date: Date): Date {
  return new Date(date.getTime() - date.getTimezoneOffset() * 60000);
}
export const localInput = (date: Date) =>
  localDate(date).toISOString().slice(0, 19);
export function utcFromLocal(value: string): string {
  const d = new Date(value);
  if (!Number.isFinite(d.getTime()) || localInput(d) !== value)
    throw new Error("존재하지 않는 로컬 시각입니다.");
  if (d.getTime() <= Date.now())
    throw new Error("미래 실행 시각을 선택하세요.");
  return d.toISOString();
}
const dateTimeFormat = new Intl.DateTimeFormat("ko-KR", {
  timeZone: "UTC",
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  hour12: false,
});
export const formatTime = (value: string) =>
  dateTimeFormat.format(localDate(new Date(value)));
export function remaining(value: string, now: number): string {
  const s = Math.ceil((new Date(value).getTime() - now) / 1000);
  if (s <= 0) return "실행 시각 도달";
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return [h ? h + "시간" : "", m ? m + "분" : "", (s % 60) + "초"]
    .filter(Boolean)
    .join(" ");
}
export const labels: Record<string, string> = {
  pending: "예약 대기",
  launching: "터미널 여는 중",
  launched: "실행 중",
  completed: "완료",
  failed: "실패",
  canceled: "취소",
  missed: "놓친 예약",
  unknown: "결과 확인 중",
  tracking_unavailable: "종료 추적 불가",
};
