// Find the axis a controller rolls around ("twist it like a screwdriver") - a clean measurement of how YOU hold it.
//
//     node tools/calibrate_pointing_axis.ts
//
// Hold the controller aimed at the screen; after 10 s it records for 10 s while you twist it around its pointing direction
// without changing where it points. Prints the axis in the controller frame; add it to CANDIDATES in the bridge to use it.
import { setTimeout as sleep } from "node:timers/promises";

import type { Mat3, Vec3 } from "#/math.ts";
import { OpenVr } from "#/openvr.ts";

const WARMUP_MS = 10_000;
const RECORD_MS = 10_000;
const POLL_MS = 10;
const MIN_ROTATION = 1e-4;
const POWER_ITERATIONS = 200;
const AXIS_DECIMALS = 3;
const PURITY_DECIMALS = 2;
const HALF = 0.5;
const AXES: readonly number[] = [0, 1, 2];
const SIZE = AXES.length;

function isSameRotation({ first, second }: { first: Mat3; second: Mat3 }) {
  return first.every((row, i) => row.every((value, j) => value === second[i]?.[j]));
}

/** The rotation from `prev` to `cur`, expressed in the controller own frame, as a small-angle rotation vector. */
function relativeRotation({ prev, cur }: { prev: Mat3; cur: Mat3 }) {
  function delta({ row, column }: { row: number; column: number }) {
    return AXES.reduce((sum, axis) => sum + (prev[axis]?.[row] ?? 0) * (cur[axis]?.[column] ?? 0), 0);
  }
  return [
    (delta({
      row: 2,
      column: 1
    }) -
      delta({
        row: 1,
        column: 2
      })) *
      HALF,
    (delta({
      row: 0,
      column: 2
    }) -
      delta({
        row: 2,
        column: 0
      })) *
      HALF,
    (delta({
      row: 1,
      column: 0
    }) -
      delta({
        row: 0,
        column: 1
      })) *
      HALF
  ] as const satisfies Vec3;
}

/** Row-major 3x3 sum of outer products; the dominant direction of the samples is the twist axis. */
function addOuterProduct({ covariance, vector }: { covariance: number[]; vector: Vec3 }) {
  for (const i of AXES) {
    for (const j of AXES) {
      covariance[i * SIZE + j] = (covariance[i * SIZE + j] ?? 0) + vector[i] * vector[j];
    }
  }
}

function multiply({ matrix, vector }: { matrix: readonly number[]; vector: readonly number[] }) {
  return AXES.map(i => AXES.reduce((sum, j) => sum + (matrix[i * SIZE + j] ?? 0) * (vector[j] ?? 0), 0));
}

function normalized(vector: readonly number[]) {
  const length = Math.hypot(...vector);
  return length === 0 ? vector : vector.map(value => value / length);
}

/** Dominant eigenvector and eigenvalue of a symmetric positive semi-definite 3x3 matrix, by power iteration. */
function dominantEigen(covariance: readonly number[]) {
  let unit = normalized(
    multiply({
      matrix: covariance,
      vector: [1, 1, 1]
    })
  );
  for (let step = 0; step < POWER_ITERATIONS; step++) {
    unit = normalized(
      multiply({
        matrix: covariance,
        vector: unit
      })
    );
  }
  const eigenvalue = multiply({
    matrix: covariance,
    vector: unit
  }).reduce((sum, value, i) => sum + value * (unit[i] ?? 0), 0);
  return {
    unit,
    eigenvalue
  };
}

const openVr = new OpenVr();
const hand = openVr.controllerIndex("right");
if (hand === openVr.invalid) {
  console.error("no right-hand controller: SteamVR does not see it (is it awake and tracked?)");
  process.exit(1);
}

function rotation() {
  const pose = openVr.poses()(hand);
  if (!pose) {
    throw new Error("controller pose unavailable");
  }

  return pose.rot;
}

await sleep(WARMUP_MS);
console.log("recording - twist now");
const covariance = Array.from({ length: SIZE * SIZE }, () => 0);
let samples = 0;
let prev = rotation();
const endMs = Date.now() + RECORD_MS;
while (Date.now() < endMs) {
  await sleep(POLL_MS);
  const cur = rotation();
  if (
    isSameRotation({
      first: cur,
      second: prev
    })
  ) {
    continue;
  }

  const skew = relativeRotation({
    prev,
    cur
  });
  if (Math.hypot(...skew) > MIN_ROTATION) {
    addOuterProduct({
      covariance,
      vector: skew
    });
    samples++;
  }

  prev = cur;
}

const trace = AXES.reduce((sum, i) => sum + (covariance[i * SIZE + i] ?? 0), 0);
if (samples === 0 || trace === 0) {
  console.error("no rotation recorded - is the controller tracked and moving?");
  process.exit(1);
}

const { unit, eigenvalue } = dominantEigen(covariance);
const axis = unit.map(value => Number(value.toFixed(AXIS_DECIMALS)) + 0).join(" ");
console.log(
  `${samples} samples | axis [${axis}] (sign is arbitrary) | purity ${(eigenvalue / trace).toFixed(PURITY_DECIMALS)} (1.0 = a perfectly clean twist)`
);
