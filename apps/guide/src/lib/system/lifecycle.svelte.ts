// Where a program is between stopped and running, so the app can show a start or stop while it
// happens instead of jumping: a start asked for shows "starting" until the program is seen
// running (or a timeout passes), and the same for stopping.

export const Phase = {
  stopped: "stopped",
  starting: "starting",
  running: "running",
  stopping: "stopping"
} as const;

export type Phase = (typeof Phase)[keyof typeof Phase];

const DEFAULT_TIMEOUT_MS = 60_000;

export class Lifecycle {
  phase = $state<Phase>(Phase.stopped);
  #deadline = 0;

  get isRunning() {
    return this.phase === Phase.running;
  }

  /** Starting or stopping: a change was asked for and has not happened yet. */
  get isBusy() {
    return this.phase === Phase.starting || this.phase === Phase.stopping;
  }

  /** A start (`isStarting`) or stop was asked for: shown until the program answers. */
  expect(isStarting: boolean, timeoutMs = DEFAULT_TIMEOUT_MS) {
    this.phase = isStarting ? Phase.starting : Phase.stopping;
    this.#deadline = Date.now() + timeoutMs;
  }

  /** What the program is doing now. A pending change holds until it happens or times out. */
  observe(isUp: boolean) {
    const hasArrived = (this.phase === Phase.starting && isUp) || (this.phase === Phase.stopping && !isUp);
    if (this.isBusy && !hasArrived && Date.now() < this.#deadline) {
      return;
    }

    this.phase = isUp ? Phase.running : Phase.stopped;
  }
}
