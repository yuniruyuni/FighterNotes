const declarationPath = `${import.meta.dir}/../crates/wasm-bridge/pkg/wasm_bridge.d.ts`;
const declarationFile = Bun.file(declarationPath);

if (!(await declarationFile.exists())) {
  throw new Error(`WASM declaration is missing: ${declarationPath}`);
}

const source = await declarationFile.text();

const expectedExports = [
  "Analyzer",
  "SpatialWindowAnalyzer",
  "hp_parallelogram_json",
  "inspect_attack_info",
  "inspect_drive",
  "inspect_frame",
  "inspect_hp",
  "inspect_input",
  "inspect_super",
  "wasm_memory",
];

const expectedAnalyzerMethods = [
  "analyze_input_inplace(full_width: number, _full_height: number, _video_frame: number): void;",
  "analyze_meter_inplace(full_width: number, full_height: number, video_frame: number): void;",
  "analyze_attack_info_inplace(full_width: number, video_frame: number): void;",
  "constructor(own_side: string);",
  "finish(): string;",
  "finish_meter_timeline(): string;",
  "get_features_json(): string;",
  "get_attack_info_json(): string;",
  "apply_hp_score_counts(counts: Uint32Array): void;",
  "use_gpu_hp_scores(): void;",
  "use_gpu_hp_columns(): void;",
  "use_gpu_drive(): void;",
  "apply_drive_columns(first_frame: number, columns: Uint32Array): void;",
  "drive_columns_from_strip(side: string): Uint8Array;",
  "static drive_column_scan(side: string): Uint32Array;",
  "apply_hp_columns(first_frame: number, columns: Uint32Array): void;",
  "push_hp_score_counts(first_frame: number, counts: Uint32Array): void;",
  "hp_columns_from_strip(side: string): Uint8Array;",
  "static hp_column_scan(side: string): Uint32Array;",
  "static hsv_sv_table(): Float32Array;",
  "static channel_norm_table(): Float32Array;",
  "static hp_score_rois(): Uint32Array;",
  "static hp_score_table(): Uint8Array;",
  "set_attack_info_json(observations_json: string): void;",
  "get_fight_markers_json(): string;",
  "get_regression_events_json(): string;",
  "get_spatial_windows_json(): string;",
  "get_timeline(): string;",
  "get_tracked_inputs(): string;",
  "hud_buf_len(): number;",
  "hud_buf_ptr(): number;",
  "super_buf_ptr(): number;",
  "super_buf_len(): number;",
  "input_buf_len(): number;",
  "input_buf_ptr(): number;",
  "meter_buf_len(): number;",
  "meter_buf_ptr(): number;",
  "progress(): number;",
  "push_hud_features_inplace(full_width: number, full_height: number, video_frame: number): void;",
  "refine_with_spatial(observations_json: string): string;",
  "set_analysis_context(context_json: string): void;",
  "set_characters(own_char: string, opponent_char: string): void;",
  "set_meter_timeline(timeline_json: string): void;",
];

const expectedSpatialMethods = [
  "constructor(width: number, height: number, training_overlay: boolean);",
  "get_observations_json(): string;",
  "observe_inplace(frame_index: number, p1_teleport: boolean, p2_teleport: boolean, p1_airborne: boolean, p2_airborne: boolean): void;",
  "reset_window(): void;",
  "rgba_buf_len(): number;",
  "rgba_buf_ptr(): number;",
];

const expectedFunctions = [
  "export function hp_parallelogram_json(): string;",
  "export function inspect_attack_info(rgba: Uint8Array, width: number, height: number): string;",
  "export function inspect_drive(rgba: Uint8Array, width: number, height: number): string;",
  "export function inspect_frame(rgba: Uint8Array, width: number, height: number): string;",
  "export function inspect_hp(rgba: Uint8Array, width: number, height: number): string;",
  "export function inspect_input(rgba: Uint8Array, width: number, height: number): string;",
  "export function inspect_super(rgba: Uint8Array, width: number, height: number): string;",
  "export function wasm_memory(): any;",
];

function sorted(values: string[]): string[] {
  return values.toSorted((left, right) => left.localeCompare(right));
}

function classMethods(name: string): string[] {
  const start = source.indexOf(`export class ${name} {`);
  const end = source.indexOf("\n}", start);
  if (start < 0 || end < 0) throw new Error(`Missing WASM class: ${name}`);
  return source
    .slice(start, end)
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        /^(?:static )?(?:constructor|\w+)\(.*\)(?:: .+)?;$/.test(line) &&
        line !== "free(): void;",
    );
}

function assertEqual(
  actual: string[],
  expected: string[],
  label: string,
): void {
  const normalizedActual = sorted(actual);
  const normalizedExpected = sorted(expected);
  if (JSON.stringify(normalizedActual) !== JSON.stringify(normalizedExpected)) {
    throw new Error(
      `${label} changed\nexpected: ${normalizedExpected.join("\n")}\nactual: ${normalizedActual.join("\n")}`,
    );
  }
}

const actualExports = [...source.matchAll(/^export (?:class|function) (\w+)/gm)]
  .map((match) => match[1])
  .filter((name) => name !== "initSync");

assertEqual(actualExports, expectedExports, "WASM exports");
assertEqual(
  classMethods("Analyzer"),
  expectedAnalyzerMethods,
  "Analyzer methods",
);
assertEqual(
  classMethods("SpatialWindowAnalyzer"),
  expectedSpatialMethods,
  "SpatialWindowAnalyzer methods",
);

for (const declaration of expectedFunctions) {
  if (!source.includes(declaration))
    throw new Error(`WASM function changed: ${declaration}`);
}
