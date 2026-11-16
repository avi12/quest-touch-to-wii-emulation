import { BUTTONS_1, BUTTONS_2, stickByte } from "#/dsu_server.ts";
import type { PadState, Quad } from "#/dsu_server.ts";
import type { Sensors } from "#/motion.ts";
import type { ControllerState } from "#/openvr.ts";
import type { PointerPosition } from "#/pointer.ts";

const PRESS_THRESHOLD = 0.5; // analog trigger / grip counts as a press above this
const STICK_THRESHOLD = 0.6; // right stick -> D-pad (the stick axes carry the pointer)
const BYTE_MAX = 255;
const MICROSECONDS_PER_MILLISECOND = 1000;

export const NO_BUTTONS: ControllerState = {
  stick: [0, 0],
  trig: 0,
  grip: 0,
  a: false,
  b: false,
  click: false,
  sys: false
};

interface Controllers {
  right: ControllerState;
  left: ControllerState;
}
type FlagRule = readonly [boolean, number];

function combineFlags(rules: readonly FlagRule[]) {
  let flags = 0;
  for (const [isOn, flag] of rules) {
    if (isOn) {
      flags |= flag;
    }
  }

  return flags;
}

function toByte(analog: number) {
  return Math.trunc(analog * BYTE_MAX);
}

function firstButtons({ right, left, isLeftAttached }: Controllers & { isLeftAttached: boolean }) {
  const [stickX, stickY] = right.stick;
  return combineFlags([
    [stickY > STICK_THRESHOLD, BUTTONS_1.up],
    [stickY < -STICK_THRESHOLD, BUTTONS_1.down],
    [stickX > STICK_THRESHOLD, BUTTONS_1.right],
    [stickX < -STICK_THRESHOLD, BUTTONS_1.left],
    [right.click, BUTTONS_1.r3],
    [isLeftAttached, BUTTONS_1.l3], // Dolphin's Extension is bound to L3: on = Nunchuk plugged in
    [left.sys, BUTTONS_1.options],
    [right.sys, BUTTONS_1.share]
  ]);
}

function secondButtons({ right, left }: Controllers) {
  return combineFlags([
    [right.a, BUTTONS_2.a],
    [right.b, BUTTONS_2.b],
    [left.a, BUTTONS_2.x],
    [left.b, BUTTONS_2.r2], // Dolphin's Wii button 2 is bound to the analog R2 (see triggers in buildPad)
    [right.trig > PRESS_THRESHOLD, BUTTONS_2.y], // Dolphin's Wii Home is bound to Triangle
    [left.trig > PRESS_THRESHOLD, BUTTONS_2.l2],
    [right.grip > PRESS_THRESHOLD, BUTTONS_2.r1],
    [left.grip > PRESS_THRESHOLD, BUTTONS_2.l1]
  ]);
}

export function buildPad({
  right,
  left,
  isLeftAttached,
  pointer,
  sensors
}: Controllers & {
  isLeftAttached: boolean; // the left controller is switched on and tracked
  pointer: PointerPosition;
  sensors: Sensors;
}) {
  const sticks: Quad = [stickByte(left.stick[0]), stickByte(left.stick[1]), stickByte(pointer.x), stickByte(pointer.y)];
  const triggers: Quad = [toByte(right.grip), toByte(left.grip), toByte(Number(left.b)), toByte(left.trig)];
  const pad: PadState = {
    buttons1: firstButtons({
      right,
      left,
      isLeftAttached
    }),
    buttons2: secondButtons({
      right,
      left
    }),
    sticks,
    triggers,
    tsUs: Date.now() * MICROSECONDS_PER_MILLISECOND,
    accel: sensors.accel,
    gyro: sensors.gyro
  };
  return pad;
}
