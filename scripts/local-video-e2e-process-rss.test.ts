import { describe, expect, test } from "bun:test";
import {
  linuxProcessTreeRssBytes,
  startProcessTreeRssSampler,
} from "./local-video-e2e-process-rss";

describe("local video E2E process-tree RSS", () => {
  test("reads non-zero RSS for the current Linux process", async () => {
    if (process.platform !== "linux") return;

    expect(await linuxProcessTreeRssBytes(process.pid)).toBeGreaterThan(0);
  });

  test("rejects a supported sampler that never reads memory data", async () => {
    const sampler = startProcessTreeRssSampler(123, {
      platform: "linux",
      readTreeRssBytes: async () => 0,
      sleep: async () => undefined,
    });

    await expect(sampler.stop()).rejects.toThrow("no readable memory data");
    expect(sampler.peakBytes).toBe(0);
  });

  test("keeps unsupported platforms explicit and idempotently stops", async () => {
    const sampler = startProcessTreeRssSampler(123, { platform: "darwin" });

    await sampler.stop();
    await sampler.stop();
    expect(sampler.peakBytes).toBeNull();
  });
});
