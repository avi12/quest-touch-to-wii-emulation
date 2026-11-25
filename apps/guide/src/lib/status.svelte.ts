import { bridges } from "./bridges/bridges.svelte.ts";
import { isDolphinConfigured } from "./dolphin/config.ts";
import { launchWiiMenu, type Paths } from "./dolphin/launch.ts";
import { bridgeExe } from "./system/home.ts";
import { Lifecycle, Phase } from "./system/lifecycle.svelte.ts";
import { isRunning, PROCESS, startSteamVr } from "./system/processes.ts";

// What each step checks, polled while the app runs, and the programs it starts: each start is
// shown as starting until the program is seen running. Once SteamVR is up the bridges can see
// the controllers, so they start then without a click, once.

const REFRESH_MS = 1000;
const STEAMVR_START_MS = 90_000;
const DOLPHIN_START_MS = 30_000;

class Status {
  readonly steamVr = new Lifecycle();
  readonly dolphin = new Lifecycle();
  isConfigured = $state(false);
  readonly hasBridge = bridgeExe() !== null;
  #hasAutoStarted = false;

  /** Dolphin is open, or opening or closing: its config must not be touched. */
  get isDolphinOpen() {
    return this.dolphin.phase !== Phase.stopped;
  }

  async refresh() {
    this.steamVr.observe(await isRunning(PROCESS.steamVr));
    const shouldAutoStart = this.steamVr.isRunning && this.hasBridge && !this.#hasAutoStarted && !bridges.isAlive;
    if (shouldAutoStart) {
      this.#hasAutoStarted = true;
      bridges.start();
    }

    this.dolphin.observe(await isRunning(PROCESS.dolphin));
    this.isConfigured = await isDolphinConfigured();
  }

  startSteamVr() {
    this.steamVr.expect(true, STEAMVR_START_MS);
    startSteamVr();
  }

  openWiiMenu(paths: Paths) {
    this.dolphin.expect(true, DOLPHIN_START_MS);
    launchWiiMenu(paths);
  }

  /** Polls until the returned function is called. */
  watch() {
    void this.refresh();
    const interval = setInterval(() => void this.refresh(), REFRESH_MS);
    return () => clearInterval(interval);
  }
}

export const status = new Status();
