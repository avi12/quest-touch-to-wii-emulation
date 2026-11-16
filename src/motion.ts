import {
  add,
  clamp,
  degreesToRadians,
  dot,
  multiplyTransposed,
  norm,
  radiansToDegrees,
  scale,
  subtract
} from "#/math.ts";
import type { Mat3, Vec3 } from "#/math.ts";
import type { DevicePose } from "#/openvr.ts";

const GRAVITY = 9.80665;
const FLIP_ACCEL: Vec3 = [1, 1, 1]; // tweak if Dolphin shows an axis inverted
const FLIP_GYRO: Vec3 = [1, 1, 1];
const SWING_GAIN = 1.3; // swing acceleration passed to Dolphin (gated by SWING_LOW/HIGH so gentle aiming cannot bend the motion pointer's pitch)
const SWING_LOW = 0.6;
const SWING_HIGH = 1.2; // g: below LOW the motion is ignored (aiming), above HIGH it is passed in full
const MAX_SWING_G = 3; // a real Wii Remote's accelerometer saturates near 3 g; far larger values corrupt Dolphin's motion tracking
const ROLL_FADE_LOW = 0.25; // roll about the aim axis is meaningless when the aim is near vertical: gravity then has no sideways part
const ROLL_FADE_HIGH = 0.5; // sideways gravity (0..1) at and above which roll is passed in full
const ROLL_GAIN = 1.0; // 1 = hand follows the controller's roll 1:1; negative flips the direction
const WORLD_UP: Vec3 = [0, 1, 0];
const MAX_ANGULAR_RATE = degreesToRadians(1200); // faster than this is a tracking jump, not real motion
const ACCELERATION_WINDOW_SECONDS = 0.015; // velocity change is measured over at least this long
const ACCELERATION_SMOOTHING = 0.35;
const NO_MOTION: Vec3 = [0, 0, 0];

interface VelocitySample {
  at: number;
  velocity: Vec3;
}

export interface Motion {
  acceleration: Vec3;
  angularRate: Vec3;
}

export interface Sensors {
  accel: Vec3;
  gyro: Vec3;
}

export const LEVEL_STILL_SENSORS: Sensors = {
  accel: [0, -1, 0],
  gyro: [0, 0, 0]
};

/**
 * Turns SteamVR's own velocity and angular velocity into a smoothed world-frame acceleration and angular rate.
 * Acceleration is the velocity change over a window, not a step: poses are extrapolated afresh on every call (about
 * 750 a second), so differencing consecutive ones amplified noise into spikes of 20 g.
 */
export class MotionTracker {
  #history: VelocitySample[] = [];
  #linear: Vec3 = NO_MOTION;
  #angular: Vec3 = NO_MOTION;
  #smoothed: Vec3 = NO_MOTION;

  update({ pose, now }: { pose: DevicePose; now: number }) {
    this.#trackVelocity({
      velocity: pose.velocity,
      now
    });
    const isRealRotation = norm(pose.angularVelocity) <= MAX_ANGULAR_RATE; // faster is a tracking jump
    if (isRealRotation) {
      this.#angular = pose.angularVelocity;
    }

    this.#smoothed = add({
      first: scale({
        vector: this.#smoothed,
        factor: ACCELERATION_SMOOTHING
      }),
      second: scale({
        vector: this.#linear,
        factor: 1 - ACCELERATION_SMOOTHING
      })
    });
    const motion: Motion = {
      acceleration: this.#smoothed,
      angularRate: this.#angular
    };
    return motion;
  }

  #trackVelocity({ velocity, now }: { velocity: Vec3; now: number }) {
    this.#history.push({
      at: now,
      velocity
    });
    while ((this.#history[1]?.at ?? Infinity) <= now - ACCELERATION_WINDOW_SECONDS) {
      this.#history.shift();
    }

    const anchor = this.#history[0];
    const seconds = anchor ? now - anchor.at : 0;
    const hasBaseline = anchor !== undefined && seconds >= ACCELERATION_WINDOW_SECONDS / 2;
    if (!hasBaseline) {
      return;
    }

    this.#linear = scale({
      vector: subtract({
        first: velocity,
        second: anchor.velocity
      }),
      factor: 1 / seconds
    });
  }
}

/** Each axis of a real Wii Remote's accelerometer saturates on its own, so the cap is per axis, not on the total. */
function limitAxis(value: number) {
  return clamp({
    value,
    min: -MAX_SWING_G,
    max: MAX_SWING_G
  });
}

/** Controller frame -> Wiimote frame (right, forward, up) using the pointing basis. */
function toWiimoteFrame({ basis, vector }: { basis: Mat3; vector: Vec3 }) {
  const wiimoteVector: Vec3 = [
    dot({
      first: basis[0],
      second: vector
    }),
    dot({
      first: basis[1],
      second: vector
    }),
    dot({
      first: basis[2],
      second: vector
    })
  ];
  return wiimoteVector;
}

/**
 * Wiimote-frame motion in Dolphin's DSU convention (read from Dolphin's DualShockUDPClient.cpp):
 * accel (g): x = -right, y = -up, z = forward; gyro (deg/s): roll = +about forward.
 * Dolphin's motion-sensor pointer turns the hand by the Wiimote's roll, so it is fed ROLL ONLY (aim comes from the pointer):
 * gravity is replaced by a version rotated by the roll about the aim axis, yaw/pitch rates are dropped, swings stay.
 */
export function toSensors({ rotation, basis, motion }: { rotation: Mat3; basis: Mat3; motion: Motion }) {
  const { acceleration, angularRate } = motion;
  const specificForce = scale({
    vector: multiplyTransposed({
      matrix: rotation,
      vector: [acceleration[0], acceleration[1] + GRAVITY, acceleration[2]]
    }),
    factor: 1 / GRAVITY
  });
  const angularInWiimote = toWiimoteFrame({
    basis,
    vector: multiplyTransposed({
      matrix: rotation,
      vector: angularRate
    })
  });
  const rollRate = radiansToDegrees(angularInWiimote[1]);
  const gravityInWiimote = toWiimoteFrame({
    basis,
    vector: multiplyTransposed({
      matrix: rotation,
      vector: WORLD_UP
    })
  });
  const rollWeight = clamp({
    value: (Math.hypot(gravityInWiimote[0], gravityInWiimote[2]) - ROLL_FADE_LOW) / (ROLL_FADE_HIGH - ROLL_FADE_LOW),
    min: 0,
    max: 1
  });
  const roll = Math.atan2(gravityInWiimote[0], gravityInWiimote[2]) * ROLL_GAIN * rollWeight;
  const rollOnlyGravity: Vec3 = [Math.sin(roll), 0, Math.cos(roll)];
  const swing = subtract({
    first: toWiimoteFrame({
      basis,
      vector: specificForce
    }),
    second: gravityInWiimote
  });
  const swingFactor = Math.min(1, Math.max(0, (norm(swing) - SWING_LOW) / (SWING_HIGH - SWING_LOW)));
  const boosted = scale({
    vector: swing,
    factor: SWING_GAIN * swingFactor
  });
  const [right, forward, upward] = add({
    first: rollOnlyGravity,
    second: [limitAxis(boosted[0]), limitAxis(boosted[1]), limitAxis(boosted[2])]
  });
  const sensors: Sensors = {
    accel: [-right * FLIP_ACCEL[0], -upward * FLIP_ACCEL[1], forward * FLIP_ACCEL[2]],
    gyro: [0, 0, rollRate * rollWeight * (1 - swingFactor) * FLIP_GYRO[2]] // no roll rate during a swing: Dolphin would integrate it into a lasting tilt
  };
  return sensors;
}
