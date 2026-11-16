import {
  add,
  degreesToRadians,
  dot,
  multiplyTransposed,
  norm,
  radiansToDegrees,
  rotationRate,
  scale,
  subtract
} from "#/math.ts";
import type { Mat3, Vec3 } from "#/math.ts";
import type { DevicePose } from "#/openvr.ts";

const GRAVITY = 9.80665;
const FLIP_ACCEL: Vec3 = [1, 1, 1]; // tweak if Dolphin shows an axis inverted
const FLIP_GYRO: Vec3 = [1, 1, 1];
const SWING_GAIN = 1.0; // swing acceleration passed to Dolphin (gated by SWING_LOW/HIGH so gentle aiming cannot bend the motion pointer's pitch)
const SWING_LOW = 0.6;
const SWING_HIGH = 1.2; // g: below LOW the motion is ignored (aiming), above HIGH it is passed in full
const MAX_SWING_G = 3; // a real Wii Remote's accelerometer saturates near 3 g; far larger values corrupt Dolphin's motion tracking
const ROLL_GAIN = 1.0; // 1 = hand follows the controller's roll 1:1; negative flips the direction
const WORLD_UP: Vec3 = [0, 1, 0];
const MAX_ANGULAR_RATE = degreesToRadians(1200); // faster than this is a tracking jump, not real motion
const MIN_POSE_INTERVAL_SECONDS = 1e-3;
const ACCELERATION_SMOOTHING = 0.5;
const NO_MOTION: Vec3 = [0, 0, 0];

interface PoseSample {
  pos: Vec3;
  rot: Mat3;
  velocity: Vec3 | null;
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

/** Differentiates successive poses into a smoothed world-frame acceleration and angular rate. */
export class MotionTracker {
  #previous: PoseSample | null = null;
  #previousAt = 0;
  #linear: Vec3 = NO_MOTION;
  #angular: Vec3 = NO_MOTION;
  #smoothed: Vec3 = NO_MOTION;

  update({ pose, now }: { pose: DevicePose; now: number }) {
    this.#trackPose({
      pose,
      now
    });
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

  #trackPose({ pose, now }: { pose: DevicePose; now: number }) {
    const previous = this.#previous;
    if (!previous) {
      this.#previous = {
        pos: pose.pos,
        rot: pose.rot,
        velocity: null
      };
      this.#previousAt = now;
      return;
    }

    if (!pose.isNewerThan(previous)) {
      return;
    }

    const seconds = now - this.#previousAt;
    if (seconds <= MIN_POSE_INTERVAL_SECONDS) {
      return;
    }

    const velocity = scale({
      vector: subtract({
        first: pose.pos,
        second: previous.pos
      }),
      factor: 1 / seconds
    });
    const linear = previous.velocity
      ? scale({
          vector: subtract({
            first: velocity,
            second: previous.velocity
          }),
          factor: 1 / seconds
        })
      : NO_MOTION;
    const angular = rotationRate({
      current: pose.rot,
      previous: previous.rot,
      seconds
    });
    this.#previous = {
      pos: pose.pos,
      rot: pose.rot,
      velocity
    };
    this.#previousAt = now;

    if (norm(angular) > MAX_ANGULAR_RATE) {
      return;
    }

    this.#linear = linear;
    this.#angular = angular;
  }
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
  const roll = Math.atan2(gravityInWiimote[0], gravityInWiimote[2]) * ROLL_GAIN;
  const rollOnlyGravity: Vec3 = [Math.sin(roll), 0, Math.cos(roll)];
  const swing = subtract({
    first: toWiimoteFrame({
      basis,
      vector: specificForce
    }),
    second: gravityInWiimote
  });
  const swingFactor = Math.min(1, Math.max(0, (norm(swing) - SWING_LOW) / (SWING_HIGH - SWING_LOW)));
  const [right, forward, upward] = add({
    first: rollOnlyGravity,
    second: scale({
      vector: swing,
      factor: SWING_GAIN * swingFactor * Math.min(1, MAX_SWING_G / Math.max(norm(swing), 1e-9))
    })
  });
  const sensors: Sensors = {
    accel: [-right * FLIP_ACCEL[0], -upward * FLIP_ACCEL[1], forward * FLIP_ACCEL[2]],
    gyro: [0, 0, rollRate * FLIP_GYRO[2]]
  };
  return sensors;
}
