import { labels } from "../lib/time";
export function StatusBadge({ status }: { status: string }) {
  return <span className={"badge " + status}>{labels[status] ?? status}</span>;
}
