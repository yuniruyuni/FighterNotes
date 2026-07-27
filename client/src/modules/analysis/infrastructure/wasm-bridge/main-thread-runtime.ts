import init from "../../../../../../crates/wasm-bridge/pkg/wasm_bridge.js";

let initialization: Promise<unknown> | null = null;

export async function ensureAnalysisWasm(): Promise<void> {
  initialization ??= init().catch((error) => {
    initialization = null;
    throw error;
  });
  await initialization;
}
