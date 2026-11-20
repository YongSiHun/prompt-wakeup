import { useSyncExternalStore } from "react";
const listeners = new Set<() => void>();
let now = Date.now();
let timer: ReturnType<typeof setInterval> | undefined;
// 상태: 시계 갱신.
function tick() {
  now = Date.now();
  listeners.forEach((listener) => listener());
}
// 상태: 마지막 구독 해제 시 정리.
function unsubscribe(listener: () => void) {
  listeners.delete(listener);
  if (listeners.size === 0) {
    clearInterval(timer);
    timer = undefined;
  }
}
// 상태: 공유 타이머 시작.
function subscribe(listener: () => void) {
  listeners.add(listener);
  if (timer === undefined) {
    now = Date.now();
    timer = setInterval(tick, 1000);
  }
  return () => unsubscribe(listener);
}
// 상태: 공유 시계 구독.
export function useNow() {
  return useSyncExternalStore(subscribe, () => now);
}
