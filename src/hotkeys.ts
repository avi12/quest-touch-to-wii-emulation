import { isKeyDown, KEY } from "#/openvr.ts";

interface HotkeyEvents {
  isLeftToggled: boolean;
  candidateStep: -1 | 0 | 1;
}

function toCandidateStep({ isNextPressed, isPreviousPressed }: { isNextPressed: boolean; isPreviousPressed: boolean }) {
  if (isNextPressed) {
    return 1;
  }

  return isPreviousPressed ? -1 : 0;
}

class RisingEdge {
  #wasDown = false;

  update(isDown: boolean) {
    const isEdge = isDown && !this.#wasDown;
    this.#wasDown = isDown;
    return isEdge;
  }
}

/** Global Ctrl+Alt hotkeys (never clash with Dolphin's), reported once per key press. */
export class Hotkeys {
  readonly #isPlayer2: boolean;
  readonly #toggle = new RisingEdge();
  readonly #next = new RisingEdge();
  readonly #previous = new RisingEdge();

  constructor({ isPlayer2 }: { isPlayer2: boolean }) {
    this.#isPlayer2 = isPlayer2;
  }

  poll() {
    const isModifierHeld = isKeyDown(KEY.ctrl) && isKeyDown(KEY.alt);
    const isCycleEnabled = isModifierHeld && !this.#isPlayer2;
    const isNextPressed = this.#next.update(isCycleEnabled && isKeyDown(KEY.rightBracket));
    const isPreviousPressed = this.#previous.update(isCycleEnabled && isKeyDown(KEY.leftBracket));
    const events: HotkeyEvents = {
      isLeftToggled: this.#toggle.update(isModifierHeld && isKeyDown(KEY.l)),
      candidateStep: toCandidateStep({
        isNextPressed,
        isPreviousPressed
      })
    };
    return events;
  }
}
