import { afterEach, expect } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";

GlobalRegistrator.register();
function TestVideoDecoder() {}
TestVideoDecoder.isConfigSupported = async (config: VideoDecoderConfig) => ({
  config,
  supported: true,
});
Object.defineProperty(globalThis, "VideoDecoder", {
  configurable: true,
  writable: true,
  value: TestVideoDecoder,
});
class TestWorker {}
Object.defineProperty(globalThis, "Worker", {
  configurable: true,
  writable: true,
  value: TestWorker,
});
class TestOffscreenCanvas {
  readonly width: number;
  readonly height: number;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
  }

  getContext(type: string) {
    return type === "2d" ? {} : null;
  }
}
Object.defineProperty(globalThis, "OffscreenCanvas", {
  configurable: true,
  writable: true,
  value: TestOffscreenCanvas,
});
class TestVideoFrame {
  close() {}
}
Object.defineProperty(globalThis, "VideoFrame", {
  configurable: true,
  writable: true,
  value: TestVideoFrame,
});
Object.defineProperty(globalThis, "createImageBitmap", {
  configurable: true,
  writable: true,
  value: async () => ({ width: 1, height: 1, close() {} }),
});
const matchers = await import("@testing-library/jest-dom/matchers");
const { cleanup } = await import("@testing-library/react");
expect.extend(matchers);

afterEach(() => {
  cleanup();
});
