// UI: 날짜·시분초 입력.
export function ScheduleTimeField({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const [date, time = ""] = value.split("T");
  const parts = time.split(":");
  // 검증: 시분초 숫자 입력.
  function updateTime(index: number, part: string) {
    if (!/^\d{0,2}$/.test(part)) return;
    const next = [...parts];
    next[index] = part;
    onChange(date + "T" + next.join(":"));
  }
  return (
    <fieldset className="schedule-time">
      <legend>실행 시각</legend>
      <div className="time-fields">
        <label>
          날짜
          <input
            type="date"
            required
            value={date}
            onChange={(event) => onChange(event.target.value + "T" + time)}
          />
        </label>
        {["시", "분", "초"].map((label, index) => (
          <label key={label}>
            {label}
            <input
              type="text"
              inputMode="numeric"
              required
              maxLength={2}
              pattern={index === 0 ? "([01]?[0-9]|2[0-3])" : "[0-5]?[0-9]"}
              value={parts[index] ?? ""}
              onChange={(event) => updateTime(index, event.target.value)}
            />
          </label>
        ))}
      </div>
    </fieldset>
  );
}
