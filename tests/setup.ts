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
