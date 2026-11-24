import { type ChildProcess, execFile, spawn } from "node:child_process";

import { bridgeExe, home } from "./home.ts";

// Runs touch_dsu.exe in its launcher mode (both bridges, self-update) and keeps its latest output.
// Stopping ends the tree of the one process this guide started, by its PID.

const KEPT_LINES = 6;

class Bridges {
  lines = $state<string[]>([]);
  isRunning = $state(false);
  #child: ChildProcess | null = null;

  start() {
    const exe = bridgeExe();
    if (!exe || this.#child) {
      return;
    }

    const env = { ...process.env };
    delete env["BRIDGE_HAND"];
    const child = spawn(exe, [], { cwd: home, env, windowsHide: true });
    this.#child = child;
    this.isRunning = true;
    const keep = (chunk: Buffer) => {
      const fresh = chunk.toString().split(/\r?\n/).filter(Boolean);
      this.lines = [...this.lines, ...fresh].slice(-KEPT_LINES);
    };
    child.stdout?.on("data", keep);
    child.stderr?.on("data", keep);
    child.on("exit", () => {
      this.#child = null;
      this.isRunning = false;
    });
  }

  stop() {
    const pid = this.#child?.pid;
    if (pid === undefined) {
      return;
    }

    execFile("taskkill", ["/PID", String(pid), "/T", "/F"], { windowsHide: true });
  }

  /** What the bridges last said about the controllers, in plain words. */
  get status() {
    const latest = this.lines.at(-1) ?? "";
    if (!this.isRunning) {
      return "Not running";
    }

    if (latest.includes("not tracked")) {
      return "Running, but a controller is not tracked: wake it and hold it in front of the headset";
    }

    return "Running";
  }
}

export const bridges = new Bridges();
