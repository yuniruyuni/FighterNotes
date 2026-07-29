import { describe, expect, test } from "bun:test";
import { fireEvent, render, screen } from "@testing-library/react";
import { VideoView } from "./VideoView.js";

describe("VideoView", () => {
  test("証拠動画を0.25倍・0.5倍・等速へ切り替える", () => {
    const { container } = render(
      <VideoView
        active={true}
        file={new File([new Uint8Array(1)], "local.mp4")}
        frameTimestamps={[]}
        scene={null}
        onSceneChange={() => undefined}
      />,
    );
    const video = container.querySelector("video");
    if (!video) throw new Error("video not rendered");

    expect(video.playbackRate).toBe(1);
    expect(
      screen.getByRole("button", { name: "再生速度 1倍" }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "再生速度 0.25倍" }));
    expect(video.playbackRate).toBe(0.25);
    expect(
      screen.getByRole("button", { name: "再生速度 0.25倍" }),
    ).toHaveAttribute("aria-pressed", "true");

    fireEvent.click(screen.getByRole("button", { name: "再生速度 0.5倍" }));
    expect(video.playbackRate).toBe(0.5);

    fireEvent.click(screen.getByRole("button", { name: "再生速度 1倍" }));
    expect(video.playbackRate).toBe(1);
  });
});
