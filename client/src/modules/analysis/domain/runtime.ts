export interface AnalysisRuntimeCapabilities {
  readonly secureContext: boolean;
  readonly hasVideoDecoder: boolean;
  readonly origin: string;
}

export type AnalysisRuntimeReadiness =
  | { readonly available: true }
  | { readonly available: false; readonly reason: string };

export const AnalysisRuntime = {
  evaluate(
    capabilities: AnalysisRuntimeCapabilities,
  ): AnalysisRuntimeReadiness {
    if (!capabilities.secureContext) {
      return {
        available: false,
        reason:
          "動画解析はHTTPSまたはlocalhostから開いた場合に利用できます。" +
          ` 現在の接続先: ${capabilities.origin}`,
      };
    }

    if (!capabilities.hasVideoDecoder) {
      return {
        available: false,
        reason:
          "このブラウザは動画解析に必要なWebCodecs VideoDecoderに対応していません。" +
          " ブラウザを最新版に更新するか、対応ブラウザで開いてください。",
      };
    }

    return { available: true };
  },
};
