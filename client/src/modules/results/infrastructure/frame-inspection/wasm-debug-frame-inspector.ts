import { ensureAnalysisWasm } from "~/modules/analysis/browser.js";
import {
  hp_parallelogram_json,
  inspect_attack_info,
  inspect_drive,
  inspect_frame,
  inspect_hp,
  inspect_input,
  inspect_super,
} from "../../../../../../crates/wasm-bridge/pkg/wasm_bridge.js";
import type {
  DebugAttackInfoInspection,
  DebugDriveInspection,
  DebugFrameInspector,
  DebugHpGeometry,
  DebugHpInspection,
  DebugInputInspection,
  DebugMeterInspection,
  DebugSuperInspection,
} from "../../application/debug-frame-inspection.js";

let hpGeometry: DebugHpGeometry | null = null;

export const wasmDebugFrameInspector: DebugFrameInspector = {
  async initialize() {
    await ensureAnalysisWasm();
    hpGeometry ??= JSON.parse(hp_parallelogram_json()) as DebugHpGeometry;
    return hpGeometry;
  },

  inspectMeter(rgba, width, height) {
    return JSON.parse(
      inspect_frame(rgba, width, height),
    ) as DebugMeterInspection;
  },

  inspectHp(rgba, width, height) {
    return JSON.parse(inspect_hp(rgba, width, height)) as DebugHpInspection;
  },

  inspectDrive(rgba, width, height) {
    return JSON.parse(
      inspect_drive(rgba, width, height),
    ) as DebugDriveInspection;
  },

  inspectSuper(rgba, width, height) {
    return JSON.parse(
      inspect_super(rgba, width, height),
    ) as DebugSuperInspection;
  },

  inspectInput(rgba, width, height) {
    return JSON.parse(
      inspect_input(rgba, width, height),
    ) as DebugInputInspection;
  },

  inspectAttackInfo(rgba, width, height) {
    return JSON.parse(
      inspect_attack_info(rgba, width, height),
    ) as DebugAttackInfoInspection;
  },
};
