import { expect, mock, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import type { ValidatedVideoInput } from "../../domain/video-preflight.js";
import { VideoFilePicker } from "./VideoFilePicker.js";

function validatedVideo(): ValidatedVideoInput {
  const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
  return {
    file,
    identity: {
      name: file.name,
      size: file.size,
      lastModified: file.lastModified,
      type: file.type,
    },
    metadataBytesRead: 4096,
    track: {
      trackId: 1,
      codec: "avc1.640028",
      codedWidth: 1920,
      codedHeight: 1080,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 0,
      framesPerSecond: 59.94,
      constantFrameRate: true,
      totalSamples: 600,
      maxSampleBytes: 1024,
      timescale: 60_000,
      duration: 600_600,
      decoderConfig: {
        codec: "avc1.640028",
        codedWidth: 1920,
        codedHeight: 1080,
      },
      codecConfig: {
        codec: "avc1.640028",
        width: 1920,
        height: 1080,
      },
    },
  };
}

test("VideoFilePicker shows checking, actionable failure, and validated metadata", () => {
  const props = {
    file: null,
    disabled: false,
    onChange: mock(() => undefined),
  };
  const view = render(
    <VideoFilePicker {...props} preflight={{ status: "checking" }} />,
  );
  expect(
    document.querySelector('[data-video-preflight-status="checking"]'),
  ).toHaveTextContent("確認中");
  expect(document.querySelector("#file-input")).toHaveAttribute(
    "accept",
    ".mp4,video/mp4",
  );

  view.rerender(
    <VideoFilePicker
      {...props}
      preflight={{
        status: "invalid",
        code: "variable_frame_rate",
        message: "固定60fps（CFR）で録画し直してください。",
      }}
    />,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("固定60fps（CFR）");
  expect(screen.getByRole("alert")).toHaveAttribute(
    "data-video-preflight-status",
    "invalid",
  );

  view.rerender(
    <VideoFilePicker
      {...props}
      preflight={{ status: "valid", video: validatedVideo() }}
    />,
  );
  const validStatus = document.querySelector(
    '[data-video-preflight-status="valid"]',
  );
  expect(validStatus).toHaveTextContent("MP4 / 1920×1080 / 59.94fps CFR");
  expect(validStatus).toHaveAttribute("data-video-preflight-status", "valid");
});
