import shader from "../../../../../../crates/hud-vision/shaders/strip_extract.wgsl" with {
  type: "text",
};

/** 復号フレームから strip を切り出すパス。 */
export const STRIP_EXTRACT_SHADER: string = shader;
