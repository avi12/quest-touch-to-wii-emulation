import { beep } from "#/openvr.ts";

const HOLD_MS = 1000; // a change must hold this long (ignores tracking blips)
export const CONNECTED_BEEP_HZ = 1200;
export const DISCONNECTED_BEEP_HZ = 500;
export const BEEP_MS = 200;

/** Beeps and logs when the second (left) controller connects or disconnects. */
export class SecondControllerWatcher {
  #isSeen = false;
  #changedAt = Date.now();

  update(isTracked: boolean) {
    if (isTracked === this.#isSeen) {
      this.#changedAt = Date.now();
      return;
    }

    if (Date.now() - this.#changedAt <= HOLD_MS) {
      return;
    }

    this.#isSeen = isTracked;
    beep(isTracked ? CONNECTED_BEEP_HZ : DISCONNECTED_BEEP_HZ, BEEP_MS);
    console.log("SECOND CONTROLLER", isTracked ? "CONNECTED" : "DISCONNECTED");
  }
}
