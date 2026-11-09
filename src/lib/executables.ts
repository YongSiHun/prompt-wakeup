import type { TerminalInfo } from "../types";

// Older settings store a terminal kind; newer settings store the exact file ID.
export function defaultTerminalId(
  terminals: TerminalInfo[],
  preference: string,
): string {
  return (
    terminals.find(
      (terminal) =>
        terminal.supported &&
        (terminal.id === preference || terminal.kind === preference),
    )?.id ?? ""
  );
}
