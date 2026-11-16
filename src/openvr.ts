// OpenVR (SteamVR) through its C "flat" API, called from Node with koffi. No native addon of our own:
// SteamVR ships openvr_api.dll, and each interface is reached through its function table.
import { existsSync } from "node:fs";

import koffi from "koffi";
import { z } from "zod";

import type { Mat3, Vec3 } from "#/math.ts";

const STEAM_DIRS = ["C:/Program Files (x86)/Steam", "C:/Program Files/Steam"];
const DLL_SUBPATH = "steamapps/common/SteamVR/bin/win64/openvr_api.dll";
const SYSTEM_INTERFACE = "FnTable:IVRSystem_023";
const RENDER_MODELS_INTERFACE = "FnTable:IVRRenderModels_006";
const APPLICATION_BACKGROUND = 3; // VRApplication_Background: no display, no scene
const TRACKING_STANDING = 1;
export const MAX_DEVICES = 64;
const FN_TABLE_SLOTS = 64;
const INVALID_DEVICE_INDEX = 0xffffffff;
const PROPERTY_BUFFER_BYTES = 256;
const ROLE = {
  left: 1,
  right: 2
} as const;
export type Hand = keyof typeof ROLE;
// indices into each function table (header order)
const SYSTEM_FN = {
  getDeviceToAbsoluteTrackingPose: 11,
  getTrackedDeviceIndexForControllerRole: 17,
  getControllerRoleForTrackedDeviceIndex: 18,
  getTrackedDeviceClass: 19,
  isTrackedDeviceConnected: 20,
  getStringTrackedDeviceProperty: 27,
  getControllerState: 34
} as const;
const RENDER_MODELS_FN = { getComponentState: 10 } as const;
export const PROPERTY = { renderModelName: 1003 } as const;
const BUTTON = {
  system: 0,
  applicationMenu: 1,
  a: 7,
  axis0: 32
} as const;
const AXIS_COUNT = 5;
const FLOATS_PER_AXIS = 2;
const VECTOR_FLOATS = 3;
const MATRIX_FLOATS = 12; // 3x4, row-major
const MATRIX_COLUMNS = 4;
const TRANSLATION_COLUMN = 3;

/** Windows virtual-key codes used by the hotkeys. */
export const KEY = {
  ctrl: 0x11,
  alt: 0x12,
  l: 0x4c,
  leftBracket: 0xdb,
  rightBracket: 0xdd
} as const;
const KEY_DOWN_MASK = 0x8000;

const user32 = koffi.load("user32.dll");
const kernel32 = koffi.load("kernel32.dll");
const winmm = koffi.load("winmm.dll");
const getAsyncKeyState = user32.func("uint16 __stdcall GetAsyncKeyState(int vKey)");
const winBeep = kernel32.func("int __stdcall Beep(uint32 freq, uint32 ms)");
const timeBeginPeriod = winmm.func("uint32 __stdcall timeBeginPeriod(uint32 ms)");

/** True while the key is held, anywhere on the system. */
export function isKeyDown(key: number) {
  return (getAsyncKeyState(key) & KEY_DOWN_MASK) !== 0;
}

/** Non-blocking beep. */
export function beep(...[frequencyHz, durationMs]: [number, number]) {
  return winBeep.async(frequencyHz, durationMs, () => {});
}

/** Ask Windows for finer timer ticks so a 4 ms loop really runs at 250 Hz. */
export function setTimerResolution(milliseconds: number) {
  return timeBeginPeriod(milliseconds);
}

koffi.struct("TrackedDevicePose_t", {
  m: koffi.array("float", MATRIX_FLOATS), // device-to-world
  velocity: koffi.array("float", VECTOR_FLOATS),
  angularVelocity: koffi.array("float", VECTOR_FLOATS),
  trackingResult: "int32",
  valid: "bool",
  connected: "bool"
});
const ControllerStateStruct = koffi.struct("VRControllerState_t", {
  packetNum: "uint32",
  buttonPressed: "uint64",
  buttonTouched: "uint64",
  axes: koffi.array("float", AXIS_COUNT * FLOATS_PER_AXIS)
});
koffi.struct("RenderModel_ControllerMode_State_t", { scrollWheelVisible: "bool" });
koffi.struct("RenderModel_ComponentState_t", {
  trackingToComponentRenderModel: koffi.array("float", MATRIX_FLOATS),
  trackingToComponentLocal: koffi.array("float", MATRIX_FLOATS),
  properties: "uint32"
});

/** One tracked device: validity, rotation (3x3 rows), position, and whether it moved since `prev`. */
export interface DevicePose {
  valid: boolean;
  trackingResult: number;
  rot: Mat3;
  pos: Vec3;
  isNewerThan: (prev: { pos: Vec3; rot: Mat3 }) => boolean;
}

/** Sticks, triggers and buttons of one controller. */
export interface ControllerState {
  stick: readonly [number, number];
  trig: number;
  grip: number;
  a: boolean;
  b: boolean;
  click: boolean;
  sys: boolean;
}

/** A component's transform relative to the controller's tracked pose: 3x3 rotation rows plus origin. */
export interface ComponentTransform {
  rot: Mat3;
  origin: Vec3;
}

// What the DLL hands back, checked once so a layout mismatch fails loudly instead of reading garbage.
function floatsSchema(length: number) {
  return z.instanceof(Float32Array).refine(floats => floats.length === length);
}
const rawStateSchema = z.object({
  buttonPressed: z.coerce.bigint(),
  axes: floatsSchema(AXIS_COUNT * FLOATS_PER_AXIS)
});
const rawPoseSchema = z.object({
  m: floatsSchema(MATRIX_FLOATS),
  valid: z.boolean(),
  trackingResult: z.number()
});
const rawComponentSchema = z.object({ trackingToComponentLocal: floatsSchema(MATRIX_FLOATS) });

type NativeCall<Result> = (...args: unknown[]) => Result;

function findDll() {
  const dll = STEAM_DIRS.map(dir => `${dir}/${DLL_SUBPATH}`).find(existsSync);
  if (!dll) {
    throw new Error(`openvr_api.dll not found under ${STEAM_DIRS.join(" or ")}; is SteamVR installed?`);
  }

  return dll;
}

function isSameVec({ first, second }: { first: Vec3; second: Vec3 }) {
  return first.every((value, i) => value === second[i]);
}

function isSameRot({ first, second }: { first: Mat3; second: Mat3 }) {
  return first.every((row, i) =>
    isSameVec({
      first: row,
      second: second[i] ?? row
    })
  );
}

/** A row-major 3x4 matrix split into its 3x3 rotation and its translation column. */
function splitMatrix(floats: Float32Array) {
  function floatAt(i: number) {
    return floats[i] ?? 0;
  }
  function rotationRow(row: number) {
    const start = row * MATRIX_COLUMNS;
    return [floatAt(start), floatAt(start + 1), floatAt(start + 2)] as const;
  }
  function translation() {
    return [
      floatAt(TRANSLATION_COLUMN),
      floatAt(MATRIX_COLUMNS + TRANSLATION_COLUMN),
      floatAt(2 * MATRIX_COLUMNS + TRANSLATION_COLUMN)
    ] as const;
  }
  const rot: Mat3 = [rotationRow(0), rotationRow(1), rotationRow(2)];
  return {
    rot,
    origin: translation()
  };
}

function decodeText({ buffer, length }: { buffer: Buffer; length: number }) {
  return length > 1 ? buffer.toString("utf8", 0, length - 1) : "";
}

function bindNative({ table, slot, signature }: { table: unknown[]; slot: number; signature: string }) {
  const proto = koffi.proto(signature);
  function call(...args: unknown[]) {
    return koffi.call(table[slot], proto, ...args);
  }
  return call;
}

export class OpenVr {
  readonly invalid = INVALID_DEVICE_INDEX;
  readonly #getPoses: NativeCall<unknown>;
  readonly #getRole: NativeCall<number>;
  readonly #getRoleOf: NativeCall<number>;
  readonly #getClass: NativeCall<number>;
  readonly #isConnected: NativeCall<boolean>;
  readonly #getText: NativeCall<number>;
  readonly #getState: NativeCall<boolean>;
  readonly #getComponentState: NativeCall<boolean>;

  constructor() {
    const lib = koffi.load(findDll());
    const initInternal = lib.func("intptr VR_InitInternal2(_Out_ int *error, int type, const char *startupInfo)");
    const getInterface = lib.func("void *VR_GetGenericInterface(const char *version, _Out_ int *error)");
    const describeError = lib.func("const char *VR_GetVRInitErrorAsSymbol(int error)");
    const error: number[] = [0];
    initInternal(error, APPLICATION_BACKGROUND, null);

    if (error[0] !== 0) {
      throw new Error(`OpenVR init failed: ${describeError(error[0])} (${error[0]})`);
    }

    function tableOf(name: string) {
      const table = getInterface(name, error);
      if (error[0] !== 0 || !table) {
        throw new Error(`OpenVR ${name} unavailable (${error[0]})`);
      }

      return koffi.decode(table, koffi.array("void *", FN_TABLE_SLOTS));
    }

    const system = tableOf(SYSTEM_INTERFACE);
    function bindSystem({ slot, signature }: { slot: number; signature: string }) {
      return bindNative({
        table: system,
        slot,
        signature
      });
    }
    this.#getPoses = bindSystem({
      slot: SYSTEM_FN.getDeviceToAbsoluteTrackingPose,
      signature: "void GetPoses(int origin, float secondsAhead, _Out_ TrackedDevicePose_t *poses, uint32 count)"
    });
    this.#getRole = bindSystem({
      slot: SYSTEM_FN.getTrackedDeviceIndexForControllerRole,
      signature: "uint32 GetIndexForRole(int role)"
    });
    this.#getRoleOf = bindSystem({
      slot: SYSTEM_FN.getControllerRoleForTrackedDeviceIndex,
      signature: "int GetRoleForIndex(uint32 index)"
    });
    this.#getClass = bindSystem({
      slot: SYSTEM_FN.getTrackedDeviceClass,
      signature: "int GetClass(uint32 index)"
    });
    this.#isConnected = bindSystem({
      slot: SYSTEM_FN.isTrackedDeviceConnected,
      signature: "bool IsConnected(uint32 index)"
    });
    this.#getText = bindSystem({
      slot: SYSTEM_FN.getStringTrackedDeviceProperty,
      signature: "uint32 GetText(uint32 index, int property, _Out_ char *buffer, uint32 size, _Out_ int *error)"
    });
    this.#getState = bindSystem({
      slot: SYSTEM_FN.getControllerState,
      signature: "bool GetState(uint32 index, _Out_ VRControllerState_t *state, uint32 size)"
    });
    this.#getComponentState = bindNative({
      table: tableOf(RENDER_MODELS_INTERFACE),
      slot: RENDER_MODELS_FN.getComponentState,
      signature:
        "bool GetComponentState(const char *model, const char *component, VRControllerState_t *state, RenderModel_ControllerMode_State_t *mode, _Out_ RenderModel_ComponentState_t *out)"
    });
  }

  /** Snapshot of every device's pose; each one is decoded (and checked) only when asked for. */
  poses() {
    const raw: unknown[] = Array.from({ length: MAX_DEVICES });
    this.#getPoses(TRACKING_STANDING, 0, raw, MAX_DEVICES);
    return (index: number) => {
      if (index >= MAX_DEVICES) {
        return null;
      }

      const { m, valid, trackingResult } = rawPoseSchema.parse(raw[index]);
      const { rot, origin: pos } = splitMatrix(m);
      return {
        valid,
        trackingResult,
        rot,
        pos,
        isNewerThan: (prev: { pos: Vec3; rot: Mat3 }) =>
          !(
            isSameVec({
              first: pos,
              second: prev.pos
            }) &&
            isSameRot({
              first: rot,
              second: prev.rot
            })
          )
      };
    };
  }

  controllerIndex(hand: Hand) {
    return this.#getRole(ROLE[hand]);
  }

  isConnected(index: number) {
    return this.#isConnected(index);
  }

  deviceClass(index: number) {
    return this.#getClass(index);
  }

  /** 0 = none, 1 = left hand, 2 = right hand, as in SteamVR's ETrackedControllerRole. */
  controllerRole(index: number) {
    return this.#getRoleOf(index);
  }

  /** A string property of one device (for example `PROPERTY.renderModelName`), or "" when it has none. */
  stringProperty({ index, property }: { index: number; property: number }) {
    const buffer = Buffer.alloc(PROPERTY_BUFFER_BYTES);
    const length = this.#getText(index, property, buffer, PROPERTY_BUFFER_BYTES, [0]);
    return decodeText({
      buffer,
      length
    });
  }

  /** Sticks, triggers and buttons of one controller, or null when SteamVR has no state for it. */
  controllerState(index: number) {
    const raw: unknown = {};
    if (!this.#getState(index, raw, koffi.sizeof(ControllerStateStruct))) {
      return null;
    }

    const { buttonPressed, axes } = rawStateSchema.parse(raw);
    function isDown(bit: number) {
      return (buttonPressed & (1n << BigInt(bit))) !== 0n;
    }
    const [stickX = 0, stickY = 0, trig = 0, , grip = 0] = axes;
    return {
      stick: [stickX, stickY] as const,
      trig,
      grip,
      a: isDown(BUTTON.a),
      b: isDown(BUTTON.applicationMenu),
      click: isDown(BUTTON.axis0),
      sys: isDown(BUTTON.system)
    };
  }

  /** Where a render-model component (for example "tip") sits relative to the controller's tracked pose, or null when it has none. */
  componentTransform({ index, model, component }: { index: number; model: string; component: string }) {
    const rawState: unknown = {};
    if (!this.#getState(index, rawState, koffi.sizeof(ControllerStateStruct))) {
      return null;
    }

    const rawComponent: unknown = {};
    if (!this.#getComponentState(model, component, rawState, { scrollWheelVisible: false }, rawComponent)) {
      return null;
    }

    return splitMatrix(rawComponentSchema.parse(rawComponent).trackingToComponentLocal);
  }
}
