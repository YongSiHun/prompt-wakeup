import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

Object.defineProperties(HTMLDialogElement.prototype, {
  showModal: {
    value(this: HTMLDialogElement) {
      this.setAttribute("open", "");
    },
    configurable: true,
  },
  close: {
    value(this: HTMLDialogElement) {
      this.removeAttribute("open");
    },
    configurable: true,
  },
});
