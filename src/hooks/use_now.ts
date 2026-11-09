import { useSyncExternalStore } from "react";
const listeners = new Set<() => void>();
let now = Date.now();
let timer: ReturnType<typeof setInterval> | undefined;
function tick() {
  now = Date.now();
  listeners.forEach((listener) => listener());
}
function unsubscribe(listener: () => void) {
  listeners.delete(listener);
  if (listeners.size === 0) {
    clearInterval(timer);
    timer = undefined;
  }
}
function subscribe(listener: () => void) {
  listeners.add(listener);
  if (timer === undefined) {
    now = Date.now();
    timer = setInterval(tick, 1000);
  }
  return () => unsubscribe(listener);
}
export function useNow() {
  return useSyncExternalStore(subscribe, () => now);
}
