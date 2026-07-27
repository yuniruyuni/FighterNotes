export type VideoDecoderSupportChecker = (
  config: VideoDecoderConfig,
) => Promise<{ readonly supported?: boolean }>;

export async function requireSupportedVideoDecoder(
  config: VideoDecoderConfig,
  check: VideoDecoderSupportChecker = (candidate) =>
    VideoDecoder.isConfigSupported(candidate),
): Promise<void> {
  let supported = false;
  try {
    supported = (await check(config)).supported === true;
  } catch {
    // Invalid and unsupported configurations are presented identically to users.
  }

  if (!supported) {
    throw new Error(
      `この動画の映像形式（${config.codec}）は、このブラウザのWebCodecsでデコードできません。` +
        " 対応するMP4動画へ変換してから再試行してください。",
    );
  }
}
