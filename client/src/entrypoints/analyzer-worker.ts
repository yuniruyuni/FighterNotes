import { installAnalyzerWorker } from "~/modules/analysis/worker.js";

installAnalyzerWorker(self as DedicatedWorkerGlobalScope);
