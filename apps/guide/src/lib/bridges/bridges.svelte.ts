import { type ChildProcess, execFile, spawn } from "node:child_process";

import { bridgeExe, home } from "../system/home.ts";
import { Lifecycle, Phase } from "../system/lifecycle.svelte.ts";

// Runs touch_dsu.exe in its launcher mode (both bridges, self-update) and keeps its latest output.
// Stopping ends the tree of the one process this app started, by its PID. Starting lasts until
// the bridges first report, stopping until the process has exited.

const KEPT_LINES = 6;
const START_TIMEOUT_MS = 15_000;
const STOP_TIMEOUT_MS = 10_000;

class Bridges {
  readonly lifecycle = new Lifecycle();
  lines = $state<string[]>([]);
  #child: ChildProcess | null = null;

  /** The process is alive (starting, running or stopping). */
  get isAlive() {
    return this.lifecycle.phase !== Phase.stopped;
  }

  start() {
    const exe = bridgeExe();
    if (!exe || this.#child) {
      return;
    }

    const env = { ...process.env };
    delete env["BRIDGE_HAND"];
    const child = spawn(exe, [], { cwd: home, env, windowsHide: true });
    this.#child = child;
    this.lifecycle.expect(true, START_TIMEOUT_MS);
    // Silent bridges still count as running once the timeout passes, if the process is alive.
    setTimeout(() => this.lifecycle.observe(this.#child === child), START_TIMEOUT_MS);
    const keep = (chunk: Buffer) => {
      const fresh = chunk.toString().split(/\r?\n/).filter(Boolean);
      this.lines = [...this.lines, ...fresh].slice(-KEPT_LINES);
      if (this.lifecycle.phase === Phase.starting) {
        this.lifecycle.observe(true);
      }
    };
    child.stdout?.on("data", keep);
    child.stderr?.on("data", keep);
    child.on("exit", () => {
      this.#child = null;
      this.lifecycle.expect(false, 0);
      this.lifecycle.observe(false);
    });
  }

  stop() {
    const pid = this.#child?.pid;
    if (pid === undefined) {
      return;
    }

    this.lifecycle.expect(false, STOP_TIMEOUT_MS);
    execFile("taskkill", ["/PID", String(pid), "/T", "/F"], { windowsHide: true });
  }

  /** What the bridges are doing, in plain words. */
  get status() {
    const latest = this.lines.at(-1) ?? "";
    switch (this.lifecycle.phase) {
      case Phase.stopped:
        return "Not running";
      case Phase.starting:
        return "Starting…";
      case Phase.stopping:
        return "Stopping…";
      default:
        return latest.includes("not tracked")
          ? "Running, but a controller is not tracked: wake it and hold it in front of the headset"
          : "Running";
    }
  }
}

export const bridges = new Bridges();
