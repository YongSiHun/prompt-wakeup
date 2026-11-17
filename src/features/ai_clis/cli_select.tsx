import { useMemo } from "react";
import type { CliInfo, CliKind } from "../../types";
import { availableClis } from "./cli_service";
export function CliSelect({
  clis,
  value,
  path,
  onChange,
  disabled,
  required,
  emptyLabel = "AI CLI를 선택하세요",
}: {
  clis: CliInfo[];
  value: CliKind | "";
  path?: string;
  onChange: (kind: CliKind | "") => void;
  disabled?: boolean;
  required?: boolean;
  emptyLabel?: string;
}) {
  const choices = useMemo(() => availableClis(clis), [clis]);
  const existing = value && path && !choices.some((cli) => cli.kind === value);
  const selected =
    existing || choices.some((cli) => cli.kind === value) ? value : "";
  return (
    <select
      required={required}
      disabled={disabled}
      value={selected}
      title={
        path || choices.find((cli) => cli.kind === selected)?.executable_path
      }
      onChange={(event) => onChange(event.target.value as CliKind | "")}
    >
      <option value="">{emptyLabel}</option>
      {existing && (
        <option value={value}>
          {value} · {path}
        </option>
      )}
      {choices.map((cli) => (
        <option key={cli.kind} value={cli.kind}>
          {cli.kind} · {cli.kind === value && path ? path : cli.executable_path}
        </option>
      ))}
    </select>
  );
}
