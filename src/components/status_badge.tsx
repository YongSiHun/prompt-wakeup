import { labels } from "../lib/time";
// UI: 상태 표시.
export function StatusBadge({ status }: { status: string }) {
  return <span className={"badge " + status}>{labels[status] ?? status}</span>;
}
