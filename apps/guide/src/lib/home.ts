import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

// Where the guide finds touch_dsu.exe and the Dolphin config: beside its own exe in a release,
// or the repository root when run from source.

const CONFIG_MARKER = join("config", "WiimoteNew.ini");
const BRIDGE_EXE = "touch_dsu.exe";

function findHome() {
  const candidates = [dirname(process.execPath), process.cwd(), resolve(process.cwd(), "..", "..")];
  return candidates.find(folder => existsSync(join(folder, CONFIG_MARKER))) ?? process.cwd();
}

export const home = findHome();
export const configFolder = join(home, "config");

/** The bridge exe: shipped beside the guide, or the repository's release build. */
export function bridgeExe() {
  const candidates = [join(home, BRIDGE_EXE), join(home, "target", "release", BRIDGE_EXE)];
  return candidates.find(path => existsSync(path)) ?? null;
}
