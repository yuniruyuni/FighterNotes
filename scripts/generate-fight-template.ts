import { dirname, resolve } from "node:path";

const SOURCE = { x: 400, y: 300, width: 1120, height: 455 } as const;
const TEMPLATE_WIDTH = 128;
const TEMPLATE_HEIGHT = 52;
const TEMPLATE_SIZE = TEMPLATE_WIDTH * TEMPLATE_HEIGHT;
const EDGE_THRESHOLD = 80;
const MIN_SAMPLE_CORRELATION = 0.5;

interface CalibrationManifest {
  readonly schemaVersion: 1;
  readonly samples: readonly {
    readonly videoPath: string;
    readonly frame: number;
  }[];
}

const manifestPath = process.argv[2];
const outputPath =
  process.argv[3] ??
  resolve(
    import.meta.dir,
    "../crates/hud-vision/src/round_start/fight_template.bin",
  );

if (!manifestPath) {
  throw new Error(
    "usage: bun scripts/generate-fight-template.ts <local-manifest.json> [output.bin]",
  );
}

const manifest = parseManifest(await Bun.file(manifestPath).json());
const samples: Uint8Array[] = [];
for (const sample of manifest.samples) {
  samples.push(await extractSample(sample.videoPath, sample.frame));
}
validateAlignment(samples);

const mean = new Uint8Array(TEMPLATE_SIZE);
for (let index = 0; index < TEMPLATE_SIZE; index += 1) {
  let sum = 0;
  for (const sample of samples) sum += sample[index] ?? 0;
  mean[index] = Math.round(sum / samples.length);
}

await Bun.write(outputPath, mean);
console.log(
  `wrote ${mean.byteLength} template bytes from ${samples.length} samples to ${outputPath}`,
);

function parseManifest(value: unknown): CalibrationManifest {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("calibration manifest schemaVersion must be 1");
  }
  if (!Array.isArray(value.samples) || value.samples.length < 2) {
    throw new Error("calibration manifest needs at least two samples");
  }
  const base = dirname(resolve(manifestPath));
  const samples = value.samples.map((sample, index) => {
    if (
      !isRecord(sample) ||
      typeof sample.videoPath !== "string" ||
      !Number.isInteger(sample.frame) ||
      (sample.frame as number) < 0
    ) {
      throw new Error(`invalid calibration sample at index ${index}`);
    }
    return {
      videoPath: resolve(base, sample.videoPath),
      frame: sample.frame as number,
    };
  });
  return { schemaVersion: 1, samples };
}

async function extractSample(
  videoPath: string,
  frame: number,
): Promise<Uint8Array> {
  const filter = [
    `select=eq(n\\,${frame})`,
    `crop=${SOURCE.width}:${SOURCE.height}:${SOURCE.x}:${SOURCE.y}`,
    `scale=${TEMPLATE_WIDTH}:${TEMPLATE_HEIGHT}:flags=area`,
    "format=gray",
  ].join(",");
  const process = Bun.spawn(
    [
      "ffmpeg",
      "-hide_banner",
      "-loglevel",
      "error",
      "-i",
      videoPath,
      "-vf",
      filter,
      "-frames:v",
      "1",
      "-f",
      "rawvideo",
      "pipe:1",
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  const [bytes, stderr, exitCode] = await Promise.all([
    new Response(process.stdout).arrayBuffer(),
    new Response(process.stderr).text(),
    process.exited,
  ]);
  if (exitCode !== 0) {
    throw new Error(
      `ffmpeg failed for ${videoPath} frame ${frame}: ${stderr.trim()}`,
    );
  }
  if (bytes.byteLength !== TEMPLATE_SIZE) {
    throw new Error(
      `${videoPath} frame ${frame} produced ${bytes.byteLength} bytes; expected ${TEMPLATE_SIZE}`,
    );
  }
  return new Uint8Array(bytes);
}

function validateAlignment(samples: readonly Uint8Array[]): void {
  const reference = gradients(samples[0] as Uint8Array);
  const mask = edgeMask(reference);
  if (mask.length < 500) {
    throw new Error(`reference has only ${mask.length} stable edge pixels`);
  }
  for (let index = 1; index < samples.length; index += 1) {
    const score = edgeCorrelation(
      reference,
      gradients(samples[index] as Uint8Array),
      mask,
    );
    if (score < MIN_SAMPLE_CORRELATION) {
      throw new Error(
        `sample ${index} is not aligned with FIGHT reference: ${score.toFixed(3)}`,
      );
    }
  }
}

interface Gradients {
  readonly x: Int16Array;
  readonly y: Int16Array;
}

function gradients(sample: Uint8Array): Gradients {
  const x = new Int16Array(TEMPLATE_SIZE);
  const y = new Int16Array(TEMPLATE_SIZE);
  for (let row = 1; row < TEMPLATE_HEIGHT - 1; row += 1) {
    for (let column = 1; column < TEMPLATE_WIDTH - 1; column += 1) {
      const index = row * TEMPLATE_WIDTH + column;
      x[index] = (sample[index + 1] ?? 0) - (sample[index - 1] ?? 0);
      y[index] =
        (sample[index + TEMPLATE_WIDTH] ?? 0) -
        (sample[index - TEMPLATE_WIDTH] ?? 0);
    }
  }
  return { x, y };
}

function edgeMask(reference: Gradients): number[] {
  const mask: number[] = [];
  for (let index = 0; index < TEMPLATE_SIZE; index += 1) {
    if (
      Math.abs(reference.x[index] ?? 0) + Math.abs(reference.y[index] ?? 0) >=
      EDGE_THRESHOLD
    ) {
      mask.push(index);
    }
  }
  return mask;
}

function edgeCorrelation(
  reference: Gradients,
  sample: Gradients,
  mask: readonly number[],
): number {
  let dot = 0;
  let referenceEnergy = 0;
  let sampleEnergy = 0;
  for (const index of mask) {
    const referenceX = reference.x[index] ?? 0;
    const referenceY = reference.y[index] ?? 0;
    const sampleX = sample.x[index] ?? 0;
    const sampleY = sample.y[index] ?? 0;
    dot += referenceX * sampleX + referenceY * sampleY;
    referenceEnergy += referenceX * referenceX + referenceY * referenceY;
    sampleEnergy += sampleX * sampleX + sampleY * sampleY;
  }
  return dot / Math.sqrt(referenceEnergy * sampleEnergy || 1);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
