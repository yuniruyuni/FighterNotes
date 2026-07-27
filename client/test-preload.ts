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
const matchers = await import("@testing-library/jest-dom/matchers");
const { cleanup } = await import("@testing-library/react");
expect.extend(matchers);

afterEach(() => {
  cleanup();
});
