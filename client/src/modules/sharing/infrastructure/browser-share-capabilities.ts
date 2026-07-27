import type {
  NativeShareData,
  ShareCapabilities,
} from "../application/ports.js";

export const BrowserShareCapabilities: ShareCapabilities = {
  async copyText(value: string): Promise<void> {
    if (!navigator.clipboard?.writeText) {
      throw new Error("clipboard unavailable");
    }
    await navigator.clipboard.writeText(value);
  },

  canShare(): boolean {
    return typeof navigator.share === "function";
  },

  async share(data: NativeShareData): Promise<void> {
    if (typeof navigator.share !== "function") {
      throw new Error("native share unavailable");
    }
    await navigator.share(data);
  },

  confirm(message: string): boolean {
    return window.confirm(message);
  },

  origin(): string {
    return window.location.origin;
  },

  isCancelledShare(error: unknown): boolean {
    return error instanceof DOMException && error.name === "AbortError";
  },
};
