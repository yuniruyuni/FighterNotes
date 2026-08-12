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

  test("キー操作で再生位置・速度・ループを動かす", () => {
    const frameTimestamps = Array.from(
      { length: 300 },
      (_, frame) => frame / 60,
    );
    const { container } = render(
      <VideoView
        active={true}
        file={new File([new Uint8Array(1)], "local.mp4")}
        frameTimestamps={frameTimestamps}
        scene={{ key: 1, frame: 120, card: null, endFrame: 180 }}
        onSceneChange={() => undefined}
      />,
    );
    const video = container.querySelector("video");
    if (!video) throw new Error("video not rendered");
    Object.defineProperty(video, "duration", { value: 5, configurable: true });
    const loop = screen.getByRole("button", { name: "区間ループ" });
    const frameOf = () => Math.round(video.currentTime * 60);

    video.currentTime = 1;
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(frameOf()).toBe(61);

    fireEvent.keyDown(window, { key: "ArrowRight", shiftKey: true });
    expect(frameOf()).toBe(71);

    fireEvent.keyDown(window, { key: "ArrowLeft", ctrlKey: true });
    expect(frameOf()).toBe(11);

    fireEvent.keyDown(window, { key: "." });
    expect(frameOf()).toBe(21);

    // 場面の先頭はループ区間の開始位置。証拠フレームの1.5秒前から始まる。
    fireEvent.keyDown(window, { key: "Home" });
    expect(frameOf()).toBe(120 - 90);

    expect(video.playbackRate).toBe(1);
    fireEvent.keyDown(window, { key: "ArrowDown" });
    expect(video.playbackRate).toBe(0.5);
    fireEvent.keyDown(window, { key: "ArrowUp" });
    expect(video.playbackRate).toBe(1);

    expect(loop).toHaveAttribute("aria-pressed", "true");
    fireEvent.keyDown(window, { key: "l" });
    expect(loop).toHaveAttribute("aria-pressed", "false");
  });

  test("再生位置sliderはコマ送りを受け取り、押せる部品からSpaceを奪わない", () => {
    const frameTimestamps = Array.from(
      { length: 300 },
      (_, frame) => frame / 60,
    );
    const { container } = render(
      <VideoView
        active={true}
        file={new File([new Uint8Array(1)], "local.mp4")}
        frameTimestamps={frameTimestamps}
        scene={null}
        onSceneChange={() => undefined}
      />,
    );
    const video = container.querySelector("video");
    if (!video) throw new Error("video not rendered");
    Object.defineProperty(video, "duration", { value: 5, configurable: true });
    video.currentTime = 1;

    const slider = screen.getByRole("slider", { name: "動画の再生位置" });
    fireEvent.keyDown(slider, { key: "ArrowRight" });
    expect(Math.round(video.currentTime * 60)).toBe(61);

    const loop = screen.getByRole("button", { name: "区間ループ" });
    const spaceOnButton = fireEvent.keyDown(loop, { key: " " });
    expect(spaceOnButton).toBe(true);
  });

  test("非表示のときはキー操作を受け取らない", () => {
    const frameTimestamps = Array.from(
      { length: 300 },
      (_, frame) => frame / 60,
    );
    const { container } = render(
      <VideoView
        active={false}
        file={new File([new Uint8Array(1)], "local.mp4")}
        frameTimestamps={frameTimestamps}
        scene={null}
        onSceneChange={() => undefined}
      />,
    );
    const video = container.querySelector("video");
    if (!video) throw new Error("video not rendered");
    Object.defineProperty(video, "duration", { value: 5, configurable: true });
    video.currentTime = 1;

    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(video.currentTime).toBe(1);
  });
});
