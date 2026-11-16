import {
  clamp,
  cross,
  degreesToRadians,
  dot,
  multiplyMatrix,
  multiplyTransposed,
  norm,
  normalize,
  OneEuro,
  scale,
  subtract
} from "#/math.ts";
import type { Mat3, Vec3 } from "#/math.ts";

const YAW_RANGE = degreesToRadians(28); // controller swing (each way) that spans the whole screen width
const PITCH_RANGE = degreesToRadians(30);
const PITCH_BIAS = 0.0; // keep 0: a recenter must put the hand exactly in the middle of the screen
const WORLD_UP: Vec3 = [0, 1, 0];
const EDGE_THRESHOLD = 0.999; // pointer value that counts as pinned at the screen edge
const EDGE_HOLD_MS = 800; // pinned and still this long: the screen is where you aim now
const STILL_RATE = degreesToRadians(20); // rad/s below which the controller counts as held still
const USABLE_AIM_LIMIT = 0.95; // aiming straight up/down has no usable screen plane
const DEFAULT_UP_HINT_LIMIT = 0.9;
const MIN_UP_LENGTH = 0.2;
const CENTER: PointerPosition = {
  x: 0,
  y: 0
};
const POINTER_LIMIT = 0.95; // Dolphin drops each IR dot that leaves its camera frame; at +-1 one dot is already on the edge
const MIN_DEPTH = 0.5; // cosine between the aim and the screen's normal below which the aim is ignored (60 degrees off); nearer the plane the projection blows up to a corner
const DEFAULT_CANDIDATE_INDEX = 7; // "Meta aim ray": measured level (7 deg) when holding still aimed at the screen; -z was 44 deg up
const SWEEP_STEP_MS = 4000;
const POINTER_SWEEP: readonly (readonly [number, number])[] = [
  [0, 0],
  [1, 0],
  [-1, 0],
  [0, 1],
  [0, -1]
];

/** Pointing axes to try live with Ctrl+Alt+[ ]: [name, axis in the controller frame] */
export const CANDIDATES: readonly (readonly [string, Vec3])[] = [
  ["measured twist axis", [-0.134, 0.991, 0.025]],
  ["+x", [1, 0, 0]],
  ["-x", [-1, 0, 0]],
  ["+y", [0, 1, 0]],
  ["-y", [0, -1, 0]],
  ["+z", [0, 0, 1]],
  ["-z", [0, 0, -1]],
  ["Meta aim ray (0,-.61,-.79)", [0, -0.607, -0.794]],
  ["Meta aim ray reversed", [0, 0.607, 0.794]]
];

export interface PointerPosition {
  x: number;
  y: number;
}

interface PointerUpdate extends PointerPosition {
  didRecenter: boolean;
  isAimedAway: boolean; // aimed too far from the screen to have a hit point; the caller keeps its last pointer
  facing: number; // cosine between the aim and the screen's normal; low or negative means aimed away from the screen
  direction: Vec3; // where the aim axis points in the world
}

/** Basis (right, forward, up) in the controller frame for a pointing axis and an "up" hint. */
function makeBasis({ forward, up }: { forward: Vec3; up: Vec3 }) {
  const basis: Mat3 = [
    cross({
      first: forward,
      second: up
    }),
    forward,
    up
  ];
  return basis;
}

function removeComponent({ vector, axis }: { vector: Vec3; axis: Vec3 }) {
  return subtract({
    first: vector,
    second: scale({
      vector: axis,
      factor: dot({
        first: vector,
        second: axis
      })
    })
  });
}

/** Any axis not parallel to the aim, made perpendicular to it. */
function defaultUp(forward: Vec3) {
  const hint: Vec3 = Math.abs(forward[1]) < DEFAULT_UP_HINT_LIMIT ? [0, 1, 0] : [0, 0, 1];
  return normalize(
    removeComponent({
      vector: hint,
      axis: forward
    })
  );
}

/** Known pointer values, ignoring the controller (PTR_TEST diagnostic). */
export function sweepPointer() {
  const [x, y] = POINTER_SWEEP[Math.floor(Date.now() / SWEEP_STEP_MS) % POINTER_SWEEP.length] ?? [0, 0];
  const position: PointerPosition = {
    x,
    y
  };
  return position;
}

/**
 * Absolute pointer from where the controller aims. Recenters on the first sample, on a thumbstick click,
 * or when pinned at a screen edge for a while.
 */
export class Pointer {
  basis: Mat3; // right, forward, up (in the controller frame)
  #aim: Vec3 = normalize(CANDIDATES[DEFAULT_CANDIDATE_INDEX]?.[1] ?? [0, 0, -1]); // pointing axis, chosen live with the Ctrl+Alt+[ ] selector; grip-dependent
  #candidateIndex = DEFAULT_CANDIDATE_INDEX;
  #screenNormal: Vec3 = [0, 0, -1];
  #screenRight: Vec3 = [1, 0, 0];
  #screenUp: Vec3 = [0, 1, 0];
  #isRecenterNeeded = true;
  #wasClickPressed = false;
  #facing = 1;
  #previousDirection: Vec3 | null = null;
  #pinnedStillSince: number | null = null;
  readonly #filterX = new OneEuro();
  readonly #filterY = new OneEuro();

  constructor() {
    this.basis = makeBasis({
      forward: this.#aim,
      up: defaultUp(this.#aim)
    });
  }

  cycleCandidate(step: number) {
    this.#candidateIndex = (this.#candidateIndex + step + CANDIDATES.length) % CANDIDATES.length;
    const [name, axis] = CANDIDATES[this.#candidateIndex] ?? ["", this.#aim];
    this.#aim = normalize(axis);
    this.#isRecenterNeeded = true;
    console.log("CANDIDATE", this.#candidateIndex, name);
  }

  update({
    rotation,
    isClickPressed,
    deltaSeconds
  }: {
    rotation: Mat3;
    isClickPressed: boolean;
    deltaSeconds: number;
  }) {
    const direction = multiplyMatrix({
      matrix: rotation,
      vector: this.#aim
    });
    if (
      this.#isPinnedAndStill({
        direction,
        deltaSeconds
      })
    ) {
      this.#isRecenterNeeded = true;
    }

    const didRecenter = this.#isRecenterNeeded || (isClickPressed && !this.#wasClickPressed);
    this.#wasClickPressed = isClickPressed;

    if (didRecenter) {
      this.#recenter({
        rotation,
        direction
      });
    }

    const update: PointerUpdate = {
      ...this.#project({
        direction,
        deltaSeconds
      }),
      didRecenter,
      isAimedAway: this.#facing < MIN_DEPTH,
      facing: this.#facing,
      direction
    };
    return update;
  }

  /** Pinned at an edge while held still means the screen is not where you aim; a shake alone must not move it. */
  #isPinnedAndStill({ direction, deltaSeconds }: { direction: Vec3; deltaSeconds: number }) {
    const previous = this.#previousDirection;
    this.#previousDirection = direction;
    const turnedBy = previous
      ? Math.acos(
          clamp({
            value:
              dot({
                first: previous,
                second: direction
              }) /
              (norm(previous) * norm(direction)),
            min: -1,
            max: 1
          })
        )
      : Infinity;
    const isStill = turnedBy / deltaSeconds < STILL_RATE;
    const isPinned =
      Math.abs(this.#filterX.value ?? 0) >= EDGE_THRESHOLD || Math.abs(this.#filterY.value ?? 0) >= EDGE_THRESHOLD;
    if (!isPinned || !isStill) {
      this.#pinnedStillSince = null;
      return false;
    }

    this.#pinnedStillSince ||= Date.now();
    const isHeldLongEnough = Date.now() - this.#pinnedStillSince > EDGE_HOLD_MS;
    const isAimUsable = Math.abs(direction[1]) < USABLE_AIM_LIMIT;
    if (!isHeldLongEnough || !isAimUsable) {
      return false;
    }

    this.#pinnedStillSince = null;
    return true;
  }

  #recenter({ rotation, direction }: { rotation: Mat3; direction: Vec3 }) {
    this.#isRecenterNeeded = false;
    this.#filterX.reset();
    this.#filterY.reset();
    this.#placeScreen(direction);
    this.#placeHand(rotation);
  }

  /** The screen is where you aim at recenter, pitch included. */
  #placeScreen(direction: Vec3) {
    const normal = normalize(direction);
    if (Math.abs(normal[1]) >= USABLE_AIM_LIMIT) {
      return;
    }

    this.#screenNormal = normal;
    this.#screenRight = normalize(
      cross({
        first: normal,
        second: WORLD_UP
      })
    );
    this.#screenUp = cross({
      first: this.#screenRight,
      second: normal
    });
  }

  /** The hand is upright at recenter, whatever the grip. */
  #placeHand(rotation: Mat3) {
    const upward = removeComponent({
      vector: multiplyTransposed({
        matrix: rotation,
        vector: WORLD_UP
      }),
      axis: this.#aim
    });
    if (norm(upward) <= MIN_UP_LENGTH) {
      return;
    }

    this.basis = makeBasis({
      forward: this.#aim,
      up: normalize(upward)
    });
  }

  /** Where the aim ray hits a flat screen. */
  #project({ direction, deltaSeconds }: { direction: Vec3; deltaSeconds: number }) {
    const length = norm(direction);
    const depth =
      dot({
        first: direction,
        second: this.#screenNormal
      }) / length;
    this.#facing = depth;
    const isBehindScreen = depth < MIN_DEPTH; // aimed away from the screen: no hit point, so the pointer stays put
    if (isBehindScreen) {
      return CENTER; // placeholder: the caller sees isAimedAway and keeps its previous pointer
    }

    const rawX =
      dot({
        first: direction,
        second: this.#screenRight
      }) /
      length /
      depth;
    const rawY =
      dot({
        first: direction,
        second: this.#screenUp
      }) /
      length /
      depth;
    const filteredX = this.#filterX.filter({
      value: rawX / Math.tan(YAW_RANGE),
      deltaSeconds
    });
    const filteredY = this.#filterY.filter({
      value: rawY / Math.tan(PITCH_RANGE),
      deltaSeconds
    });
    const position: PointerPosition = {
      x: clamp({
        value: filteredX,
        min: -POINTER_LIMIT,
        max: POINTER_LIMIT
      }),
      y: clamp({
        value: filteredY + PITCH_BIAS,
        min: -POINTER_LIMIT,
        max: POINTER_LIMIT
      })
    };
    return position;
  }
}
