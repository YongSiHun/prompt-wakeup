// 변환: 로컬 시각 보정.
function localDate(date: Date): Date {
  return new Date(date.getTime() - date.getTimezoneOffset() * 60000);
}
// 변환: 날짜→입력 문자열.
export const localInput = (date: Date) =>
  localDate(date).toISOString().slice(0, 19);
// 검증: 로컬 시각→UTC.
export function utcFromLocal(value: string): string {
  const date = new Date(value);
  if (!Number.isFinite(date.getTime()) || localInput(date) !== value)
    throw new Error("존재하지 않는 로컬 시각입니다.");
  if (date.getTime() <= Date.now())
    throw new Error("미래 실행 시각을 선택하세요.");
  return date.toISOString();
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
// 변환: 표시용 날짜·시간.
export const formatTime = (value: string) =>
  dateTimeFormat.format(localDate(new Date(value)));
// 변환: 남은 시간.
export function remaining(value: string, now: number): string {
  const seconds = Math.ceil((new Date(value).getTime() - now) / 1000);
  if (seconds <= 0) return "실행 시각 도달";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return [
    hours ? hours + "시간" : "",
    minutes ? minutes + "분" : "",
    (seconds % 60) + "초",
  ]
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
