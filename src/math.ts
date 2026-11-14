export type Vec3 = readonly [number, number, number];
export type Mat3 = readonly [Vec3, Vec3, Vec3];

interface VectorPair {
  first: Vec3;
  second: Vec3;
}

interface OneEuroOptions {
  minCutoff?: number;
  beta?: number;
  derivativeCutoff?: number;
}

const HALF_TURN_DEGREES = 180;
const MIN_SINE = 1e-12;
const DEFAULT_MIN_CUTOFF = 0.8;
const DEFAULT_BETA = 0.6;
const DEFAULT_DERIVATIVE_CUTOFF = 1.0;

function toVec3(components: Vec3) {
  return components;
}

function toMat3(rows: Mat3) {
  return rows;
}

export function degreesToRadians(degrees: number) {
  return (degrees * Math.PI) / HALF_TURN_DEGREES;
}

export function radiansToDegrees(radians: number) {
  return (radians * HALF_TURN_DEGREES) / Math.PI;
}

export function clamp({ value, min, max }: { value: number; min: number; max: number }) {
  return Math.max(min, Math.min(max, value));
}

export function dot({ first, second }: VectorPair) {
  return first[0] * second[0] + first[1] * second[1] + first[2] * second[2];
}

export function norm(vector: Vec3) {
  return Math.hypot(vector[0], vector[1], vector[2]);
}

export function scale({ vector, factor }: { vector: Vec3; factor: number }) {
  return toVec3([vector[0] * factor, vector[1] * factor, vector[2] * factor]);
}

export function normalize(vector: Vec3) {
  return scale({
    vector,
    factor: 1 / norm(vector)
  });
}

export function add({ first, second }: VectorPair) {
  return toVec3([first[0] + second[0], first[1] + second[1], first[2] + second[2]]);
}

export function subtract({ first, second }: VectorPair) {
  return toVec3([first[0] - second[0], first[1] - second[1], first[2] - second[2]]);
}

export function cross({ first, second }: VectorPair) {
  return toVec3([
    first[1] * second[2] - first[2] * second[1],
    first[2] * second[0] - first[0] * second[2],
    first[0] * second[1] - first[1] * second[0]
  ]);
}

/** matrix * vector */
export function multiplyMatrix({ matrix, vector }: { matrix: Mat3; vector: Vec3 }) {
  return toVec3([
    dot({
      first: matrix[0],
      second: vector
    }),
    dot({
      first: matrix[1],
      second: vector
    }),
    dot({
      first: matrix[2],
      second: vector
    })
  ]);
}

/** transpose(matrix) * vector */
export function multiplyTransposed({ matrix, vector }: { matrix: Mat3; vector: Vec3 }) {
  const rows = [
    scale({
      vector: matrix[0],
      factor: vector[0]
    }),
    scale({
      vector: matrix[1],
      factor: vector[1]
    }),
    scale({
      vector: matrix[2],
      factor: vector[2]
    })
  ];
  return add({
    first: add({
      first: rows[0],
      second: rows[1]
    }),
    second: rows[2]
  });
}

function multiplyByTranspose({ matrix, other }: { matrix: Mat3; other: Mat3 }) {
  const [rowX, rowY, rowZ] = matrix;
  return toMat3([
    [
      dot({
        first: rowX,
        second: other[0]
      }),
      dot({
        first: rowX,
        second: other[1]
      }),
      dot({
        first: rowX,
        second: other[2]
      })
    ],
    [
      dot({
        first: rowY,
        second: other[0]
      }),
      dot({
        first: rowY,
        second: other[1]
      }),
      dot({
        first: rowY,
        second: other[2]
      })
    ],
    [
      dot({
        first: rowZ,
        second: other[0]
      }),
      dot({
        first: rowZ,
        second: other[1]
      }),
      dot({
        first: rowZ,
        second: other[2]
      })
    ]
  ]);
}

/** World-frame angular rate (rad/s) taking rotation `previous` to `current` over `seconds`. */
export function rotationRate({ current, previous, seconds }: { current: Mat3; previous: Mat3; seconds: number }) {
  const delta = multiplyByTranspose({
    matrix: current,
    other: previous
  });
  const sineTimesAxis = scale({
    vector: [delta[2][1] - delta[1][2], delta[0][2] - delta[2][0], delta[1][0] - delta[0][1]],
    factor: 0.5
  });
  const sine = norm(sineTimesAxis);
  if (sine <= MIN_SINE) {
    return toVec3([0, 0, 0]);
  }

  return scale({
    vector: sineTimesAxis,
    factor: Math.asin(Math.min(1, sine)) / sine / seconds
  });
}

/** One-euro filter: smooths jitter when slow, stays responsive when fast. */
export class OneEuro {
  value: number | null = null;
  #derivative = 0;
  readonly #minCutoff: number;
  readonly #beta: number;
  readonly #derivativeCutoff: number;

  constructor({
    minCutoff = DEFAULT_MIN_CUTOFF,
    beta = DEFAULT_BETA,
    derivativeCutoff = DEFAULT_DERIVATIVE_CUTOFF
  }: OneEuroOptions = {}) {
    this.#minCutoff = minCutoff;
    this.#beta = beta;
    this.#derivativeCutoff = derivativeCutoff;
  }

  reset() {
    this.value = null;
    this.#derivative = 0;
  }

  static #alpha({ cutoff, deltaSeconds }: { cutoff: number; deltaSeconds: number }) {
    return 1 / (1 + 1 / (2 * Math.PI * cutoff * deltaSeconds));
  }

  filter({ value, deltaSeconds }: { value: number; deltaSeconds: number }) {
    if (this.value === null) {
      this.value = value;
      return value;
    }

    this.#derivative +=
      OneEuro.#alpha({
        cutoff: this.#derivativeCutoff,
        deltaSeconds
      }) *
      ((value - this.value) / deltaSeconds - this.#derivative);
    const cutoff = this.#minCutoff + this.#beta * Math.abs(this.#derivative);
    this.value +=
      OneEuro.#alpha({
        cutoff,
        deltaSeconds
      }) *
      (value - this.value);
    return this.value;
  }
}
