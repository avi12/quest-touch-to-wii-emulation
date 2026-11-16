// Summarise a bridge telemetry stream (see BRIDGE_TELEMETRY / scripts\start.ps1 -Telemetry).
//
//     node tools/telemetry_tail.ts [file] [seconds] [--rows] [--marks]
//
// Prints min / max / spread of the aim angles, pointer and acceleration, how long the pointer sat at an edge,
// and recentres, for the last <seconds>. With --rows it also prints one row per 250 ms.
// Holding the right controller's B button records a take: with --marks each held stretch is analysed as one recording.
import { readFileSync } from "node:fs";

import { z } from "zod";

import { CANDIDATES } from "#/pointer.ts";

const MS_PER_SECOND = 1000;
const ROW_MS = 250;
const EDGE = 0.98;
const PERCENT = 100;
const TAKE_GAP_MS = 200; // B released shorter than this still counts as the same recording
const STEEP_PITCH = 70; // aim this far above or below level: roll is ill-defined there
const JUMP = 0.25; // pointer movement (-1..1) in one 20 ms sample that is a jump, not a movement
const SWING_G = 2;
const RADIANS_TO_DEGREES = 180 / Math.PI;

const argsSchema = z.object({
  file: z.string().default("telemetry-right.jsonl"),
  seconds: z.coerce.number().positive().default(10),
  rows: z.boolean().default(false),
  marks: z.boolean().default(false)
});
const sampleSchema = z.object({
  timestamp: z.number(),
  pitch: z.number(),
  yaw: z.number(),
  pointerX: z.number(),
  pointerY: z.number(),
  accel: z.tuple([z.number(), z.number(), z.number()]),
  gyroRoll: z.number(),
  rotation: z.array(z.number()).length(9),
  facing: z.number().default(1),
  recenter: z.boolean(),
  mark: z.boolean()
});
type Sample = z.infer<typeof sampleSchema>;

function spread(values: number[]) {
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;
  const deviation = Math.sqrt(values.reduce((sum, value) => sum + (value - mean) ** 2, 0) / values.length);
  return `${Math.min(...values).toFixed(2)}..${Math.max(...values).toFixed(2)} sd ${deviation.toFixed(2)}`;
}

function printSummary({ window, label }: { window: Sample[]; label: string }) {
  const edge = window.filter(sample => Math.abs(sample.pointerX) >= EDGE || Math.abs(sample.pointerY) >= EDGE).length;
  const accelMagnitude = window.map(sample => Math.hypot(...sample.accel));
  console.log(`== ${label}: ${window.length} samples`);
  console.log(`aim pitch deg ${spread(window.map(sample => sample.pitch))}`);
  console.log(`aim yaw   deg ${spread(window.map(sample => sample.yaw))}`);
  console.log(
    `pointer x ${spread(window.map(sample => sample.pointerX))} | y ${spread(window.map(sample => sample.pointerY))}`
  );
  console.log(
    `pointer at an edge ${((edge / window.length) * PERCENT).toFixed(0)}% | recenters ${window.filter(sample => sample.recenter).length}`
  );
  console.log(
    `accel g max ${Math.max(...accelMagnitude).toFixed(2)} | roll rate deg/s ${spread(window.map(sample => sample.gyroRoll))}`
  );
}

/** Each unbroken stretch of B held is one recording. */
function splitTakes(all: Sample[]) {
  const takes: Sample[][] = [];
  let current: Sample[] = [];
  let lastHeldAt = 0;
  for (const sample of all) {
    if (!sample.mark) {
      continue;
    }

    const isNewTake = sample.timestamp - lastHeldAt > TAKE_GAP_MS;
    if (isNewTake) {
      current = [];
      takes.push(current);
    }

    current.push(sample);
    lastHeldAt = sample.timestamp;
  }

  return takes;
}

/** Plain-language findings for one recording. */
function printDiagnostics(take: Sample[]) {
  const steep = take.filter(sample => Math.abs(sample.pitch) > STEEP_PITCH).length;
  const jumps = take.filter(
    (sample, index) =>
      Math.hypot(
        sample.pointerX - (take[index - 1]?.pointerX ?? sample.pointerX),
        sample.pointerY - (take[index - 1]?.pointerY ?? sample.pointerY)
      ) > JUMP
  ).length;
  const swings = take.filter(sample => Math.hypot(...sample.accel) > SWING_G).length;
  console.log(
    `findings: aim steeper than ${STEEP_PITCH} deg ${((steep / take.length) * PERCENT).toFixed(0)}% of the time | pointer jumps >${JUMP} in 20 ms: ${jumps} | samples over ${SWING_G} g: ${swings}`
  );
}

/** Pitch (deg above level) of a controller-frame axis, from the logged rotation (row-major 3x3). */
function axisPitch({ rotation, axis }: { rotation: number[]; axis: readonly number[] }) {
  const [axisX = 0, axisY = 0, axisZ = 0] = axis;
  const [rowX = 0, rowY = 0, rowZ = 0] = rotation.slice(3, 6);
  const height = rowX * axisX + rowY * axisY + rowZ * axisZ;
  return Math.asin(Math.max(-1, Math.min(1, height))) * RADIANS_TO_DEGREES;
}

/** How steady each pointing-axis candidate would have been in this recording: smaller swing = better. */
function printAxisScores(take: Sample[]) {
  const scores = CANDIDATES.map(([name, axis]) => {
    const pitches = take.map(sample =>
      axisPitch({
        rotation: sample.rotation,
        axis
      })
    );
    return `  ${name.padEnd(28)} pitch ${spread(pitches)}`;
  });
  console.log("pointing-axis candidates (pitch swing; smaller is steadier):");
  console.log(scores.join("\n"));
}

function printRows({ window, origin }: { window: Sample[]; origin: number }) {
  let nextRow = 0;
  for (const sample of window) {
    if (sample.timestamp < nextRow) {
      continue;
    }

    nextRow = sample.timestamp + ROW_MS;
    console.log(
      `${((sample.timestamp - origin) / MS_PER_SECOND).toFixed(2)}s pitch ${sample.pitch} yaw ${sample.yaw} ptr(${sample.pointerX}, ${sample.pointerY}) accel(${sample.accel.join(", ")})`
    );
  }
}

const positional = process.argv.slice(2).filter(arg => !arg.startsWith("--"));
const parsed = argsSchema.safeParse({
  file: positional[0],
  seconds: positional[1],
  rows: process.argv.includes("--rows"),
  marks: process.argv.includes("--marks")
});
if (!parsed.success) {
  console.error(z.prettifyError(parsed.error));
  process.exit(2);
}

const { file, seconds, rows, marks } = parsed.data;
const samples = readFileSync(file, "utf8")
  .split("\n")
  .filter(Boolean)
  .flatMap(line => {
    const sample = sampleSchema.safeParse(JSON.parse(line));
    return sample.success ? [sample.data] : [];
  });
const newest = samples.at(-1);
if (!newest) {
  console.error(`no samples in ${file}`);
  process.exit(1);
}

if (marks) {
  const takes = splitTakes(samples);
  console.log(`${takes.length} recordings (B held)`);
  for (const [index, take] of takes.entries()) {
    const first = take[0]?.timestamp ?? 0;
    const takeSeconds = ((take.at(-1)?.timestamp ?? first) - first) / MS_PER_SECOND;
    printSummary({
      window: take,
      label: `recording ${index + 1}, ${takeSeconds.toFixed(1)}s`
    });
    printDiagnostics(take);
    printAxisScores(take);

    if (rows) {
      printRows({
        window: take,
        origin: first
      });
    }
  }
  process.exit(0);
}

const recent = samples.filter(sample => sample.timestamp >= newest.timestamp - seconds * MS_PER_SECOND);
printSummary({
  window: recent,
  label: `last ${seconds}s, age ${((Date.now() - newest.timestamp) / MS_PER_SECOND).toFixed(1)}s`
});

if (rows) {
  printRows({
    window: recent,
    origin: newest.timestamp
  });
}
