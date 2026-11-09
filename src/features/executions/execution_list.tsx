import { StatusBadge } from "../../components/status_badge";
import { useAppStore } from "../../stores/app_store";
import { formatTime } from "../../lib/time";
export function ExecutionList() {
  const records = useAppStore((state) => state.data!.executions);
  return (
    <>
      <h2>실행 이력</h2>
      <p className="hint">
        이력에는 프롬프트와 콘솔 원문을 저장하지 않습니다. 상세 출력은 외부
        터미널에서 확인하세요.
      </p>
      {records.length === 0 ? (
        <div className="empty">아직 실행 이력이 없습니다.</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                {["작업 / CLI", "시작 / 종료", "결과", "종료 코드 / 원인"].map(
                  (heading) => (
                    <th key={heading}>{heading}</th>
                  ),
                )}
              </tr>
            </thead>
            <tbody>
              {[...records].reverse().map((record) => (
                <tr key={record.id}>
                  <td>
                    {record.name}
                    <small>{record.cli}</small>
                  </td>
                  <td>
                    {formatTime(record.started_at)}
                    <small>
                      {record.ended_at
                        ? formatTime(record.ended_at)
                        : "종료 미확인"}
                    </small>
                  </td>
                  <td>
                    <StatusBadge status={record.status} />
                  </td>
                  <td>
                    {record.exit_code ?? "—"}
                    <small>{record.error}</small>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}
