// Optional live stream of what the bridge sees and sends, as JSON lines, so the motion can be inspected while it happens.
import { createWriteStream } from "node:fs";
import type { WriteStream } from "node:fs";

import { radiansToDegrees } from "#/math.ts";
import type { Mat3, Vec3 } from "#/math.ts";

const SAMPLE_EVERY_MS = 20; // 50 samples per second
const DECIMALS = 100;

interface TelemetrySample {
  direction: Vec3;
  pointer: {
    x: number;
    y: number;
  };
  sensors: {
    accel: Vec3;
    gyro: Vec3;
  };
  didRecenter: boolean;
  rotation: Mat3;
  isMarked: boolean; // B held: the person is recording a take to analyse
}

function round(value: number) {
  return Math.round(value * DECIMALS) / DECIMALS;
}

/** Writes one line per sample to `path`; does nothing when no path is given. */
export class Telemetry {
  readonly #stream: WriteStream | null;
  #lastWrittenAt = 0;

  constructor(path: string | undefined) {
    this.#stream = path ? createWriteStream(path, { flags: "a" }) : null;
  }

  record({ direction, pointer, sensors, didRecenter, isMarked, rotation }: TelemetrySample) {
    const now = Date.now();
    const isDue = now - this.#lastWrittenAt >= SAMPLE_EVERY_MS;
    if (!this.#stream || (!isDue && !didRecenter)) {
      return;
    }

    this.#lastWrittenAt = now;
    const [east, height, south] = direction;
    const line = {
      timestamp: now,
      pitch: round(radiansToDegrees(Math.asin(height / Math.hypot(east, height, south)))),
      yaw: round(radiansToDegrees(Math.atan2(east, -south))),
      pointerX: round(pointer.x),
      pointerY: round(pointer.y),
      accel: sensors.accel.map(round),
      gyroRoll: round(sensors.gyro[2]),
      rotation: rotation.flat().map(round),
      recenter: didRecenter,
      mark: isMarked
    };
    this.#stream.write(`${JSON.stringify(line)}\n`);
  }
}
