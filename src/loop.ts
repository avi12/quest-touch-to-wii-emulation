import { setTimeout as sleep } from "node:timers/promises";

import { buildPad, NO_BUTTONS } from "#/buttons.ts";
import { config } from "#/config.ts";
import { Server } from "#/dsu_server.ts";
import type { PadState } from "#/dsu_server.ts";
import { Hotkeys } from "#/hotkeys.ts";
import { LEVEL_STILL_SENSORS, MotionTracker, toSensors } from "#/motion.ts";
import type { Sensors } from "#/motion.ts";
import { beep, OpenVr, setTimerResolution } from "#/openvr.ts";
import type { ControllerState } from "#/openvr.ts";
import { Pointer, sweepPointer } from "#/pointer.ts";
import type { PointerPosition } from "#/pointer.ts";
import { Telemetry } from "#/telemetry.ts";
import { BEEP_MS, CONNECTED_BEEP_HZ, DISCONNECTED_BEEP_HZ, SecondControllerWatcher } from "#/tracking.ts";

const RATE_HZ = 250;
const MS_PER_SECOND = 1000;
const STEP_SECONDS = 1 / RATE_HZ;
const REPORT_INTERVAL_MS = 3000;
const RECENTER_HOLD_MS = 300;
const SWING_ACCEL_G = 1.5; // accel magnitude (g) that counts as a swing in progress
const POINTER_HOLD_MS = 200; // the pointer stays where it was during a swing and this long after it
const SWING_SETTLE_MS = 250; // calm this long after a swing, then Dolphin's motion tracking is recentred
const TIMER_RESOLUTION_MS = 1;
const DISPLAY_PRECISION = 100;
const PARKED_POINTER: PointerPosition = {
  x: -1,
  y: 0
};

interface Controllers {
  right: ControllerState;
  left: ControllerState;
}

interface Output {
  pointer: PointerPosition;
  sensors: Sensors;
}

function nowSeconds() {
  return performance.now() / MS_PER_SECOND;
}

function roundForDisplay(value: number) {
  return Math.round(value * DISPLAY_PRECISION) / DISPLAY_PRECISION;
}

class Throttle {
  #lastAt = 0;

  run(action: () => void) {
    if (Date.now() - this.#lastAt <= REPORT_INTERVAL_MS) {
      return;
    }

    action();
    this.#lastAt = Date.now();
  }
}

class Bridge {
  readonly #openVr = new OpenVr();
  readonly #server = new Server(config.port);
  readonly #motion = new MotionTracker();
  readonly #pointer = new Pointer();
  readonly #hotkeys = new Hotkeys({ isPlayer2: config.isPlayer2 });
  readonly #secondController = new SecondControllerWatcher();
  readonly #report = new Throttle();
  #isLeftEnabled = true;
  #lastPad: PadState | null = null;
  readonly #telemetry = new Telemetry(config.telemetryPath);
  #pointerFrozenUntil = 0;
  #steadyPointer: PointerPosition = {
    x: 0,
    y: 0
  };
  #lastSwingAt: number | null = null; // last moment a swing was in progress; null once Dolphin was recentred after it

  constructor() {
    setTimerResolution(TIMER_RESOLUTION_MS);
    console.log(
      "Bridge up on UDP",
      config.port,
      config.isPlayer2 ? "(left controller = Wii Remote 2)" : "(right controller = Wii Remote 1)",
      "- waiting for controllers"
    );
  }

  async run() {
    let nextTickAt = nowSeconds();
    for (;;) {
      nextTickAt += STEP_SECONDS;

      if (!this.#tick()) {
        await sleep(STEP_SECONDS * MS_PER_SECOND);
        continue;
      }

      const waitSeconds = nextTickAt - nowSeconds();
      if (waitSeconds > 0) {
        await sleep(waitSeconds * MS_PER_SECOND);
      }
    }
  }

  /** One pad update; false when the controller is not tracked. */
  #tick() {
    const poseOf = this.#openVr.poses();
    const rightIndex = this.#openVr.controllerIndex(config.hand);
    const leftIndex = config.isPlayer2 ? this.#openVr.invalid : this.#openVr.controllerIndex("left");
    const pose = poseOf(rightIndex);
    if (!pose?.valid) {
      this.#holdLastPad();
      return false;
    }

    const isLeftTracked = Boolean(poseOf(leftIndex)?.valid);
    if (!config.isPlayer2) {
      this.#secondController.update(isLeftTracked);
    }

    this.#applyHotkeys();
    const { right, left } = this.#readControllers({
      rightIndex,
      leftIndex
    });
    const motion = this.#motion.update({
      pose,
      now: nowSeconds()
    });
    const sensors = toSensors({
      rotation: pose.rot,
      basis: this.#pointer.basis,
      motion
    });
    this.#recenterMotionAfterSwing(sensors);
    const pointer = this.#pointer.update({
      rotation: pose.rot,
      isClickPressed: left.click || right.click,
      deltaSeconds: STEP_SECONDS
    });
    if (pointer.didRecenter) {
      console.log("RECENTER");
      this.#server.recenterUntil = Date.now() + RECENTER_HOLD_MS;
    }

    const output = this.#applyOverrides({
      pointer: this.#holdPointerDuringSwing(pointer),
      sensors
    });
    const pad = buildPad({
      right,
      left,
      pointer: output.pointer,
      sensors: output.sensors
    });
    this.#lastPad = {
      ...pad,
      gyro: [0, 0, 0]
    };
    this.#server.sendPad(pad);
    this.#telemetry.record({
      direction: pointer.direction,
      pointer: output.pointer,
      sensors: output.sensors,
      didRecenter: pointer.didRecenter,
      facing: pointer.facing,
      isMarked: right.b,
      rotation: pose.rot
    });
    this.#report.run(() =>
      this.#logStatus({
        isLeftTracked,
        pointer: output.pointer
      })
    );
    return true;
  }

  #holdLastPad() {
    this.#report.run(() =>
      console.log(config.hand, "controller not tracked - holding last pose (keep it in front of the headset cameras)")
    );

    if (this.#lastPad) {
      this.#server.sendPad(this.#lastPad);
    }
  }

  /** A swing sweeps the aim far past the screen, so the pointer keeps its pre-swing position until the swing is over. */
  #holdPointerDuringSwing<Pointer extends PointerPosition & { isAimedAway: boolean }>(pointer: Pointer) {
    const isFrozen = pointer.isAimedAway || Date.now() < this.#pointerFrozenUntil;
    if (!isFrozen) {
      this.#steadyPointer = {
        x: pointer.x,
        y: pointer.y
      };
    }

    const shown: Pointer = {
      ...pointer,
      ...this.#steadyPointer
    };
    return shown;
  }

  /** A hard swing leaves Dolphin's motion tracking skewed; recentre it once the controller settles. */
  #recenterMotionAfterSwing(sensors: Sensors) {
    const now = Date.now();
    const isSwinging = Math.hypot(...sensors.accel) > SWING_ACCEL_G;
    if (isSwinging) {
      this.#lastSwingAt = now;
      this.#pointerFrozenUntil = now + POINTER_HOLD_MS;
      return;
    }

    const isSettled = this.#lastSwingAt !== null && now - this.#lastSwingAt >= SWING_SETTLE_MS;
    if (!isSettled) {
      return;
    }

    this.#lastSwingAt = null;
    this.#server.recenterUntil = now + RECENTER_HOLD_MS;
  }

  #applyHotkeys() {
    const { isLeftToggled, candidateStep } = this.#hotkeys.poll();
    if (isLeftToggled) {
      this.#isLeftEnabled = !this.#isLeftEnabled;
      console.log("LEFT CONTROLLER", this.#isLeftEnabled ? "ON" : "OFF");

      if (!config.isPlayer2) {
        beep(this.#isLeftEnabled ? CONNECTED_BEEP_HZ : DISCONNECTED_BEEP_HZ, BEEP_MS); // one bridge beeps, not both
      }
    }

    if (candidateStep !== 0) {
      this.#pointer.cycleCandidate(candidateStep);
    }
  }

  #readControllers({ rightIndex, leftIndex }: { rightIndex: number; leftIndex: number }) {
    const isLeftIgnored = !this.#isLeftEnabled || leftIndex === this.#openVr.invalid;
    const isRightIgnored = config.isPlayer2 && !this.#isLeftEnabled;
    const controllers: Controllers = {
      right: isRightIgnored ? NO_BUTTONS : (this.#openVr.controllerState(rightIndex) ?? NO_BUTTONS),
      left: isLeftIgnored ? NO_BUTTONS : (this.#openVr.controllerState(leftIndex) ?? NO_BUTTONS)
    };
    return controllers;
  }

  #applyOverrides({ pointer, sensors }: Output) {
    const output: Output = config.isPointerTest
      ? {
          pointer: sweepPointer(),
          sensors: LEVEL_STILL_SENSORS
        }
      : {
          pointer,
          sensors
        };
    const isHandParked = config.isPlayer2 && !this.#isLeftEnabled;
    const parked: Output = {
      pointer: PARKED_POINTER,
      sensors: LEVEL_STILL_SENSORS // a switched-off left controller must not punch through a Nunchuk's motion
    };
    return isHandParked ? parked : output;
  }

  #logStatus({ isLeftTracked, pointer }: { isLeftTracked: boolean; pointer: PointerPosition }) {
    console.log(
      "left controller:",
      this.#isLeftEnabled ? "ON (used)" : "OFF (ignored)",
      "| seen by SteamVR:",
      isLeftTracked,
      "| dolphin connected:",
      this.#server.clients.size > 0,
      "| ptr",
      roundForDisplay(pointer.x),
      roundForDisplay(pointer.y)
    );
  }
}

export function runBridge() {
  return new Bridge().run();
}
