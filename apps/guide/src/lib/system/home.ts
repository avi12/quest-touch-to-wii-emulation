import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

// Where the app finds touch_dsu.exe, the Dolphin config and its own assets: beside its exe in a
// release, or in the repository when run from source.

const CONFIG_MARKER = join("config", "WiimoteNew.ini");
const BRIDGE_EXE = "touch_dsu.exe";
const exeFolder = dirname(process.execPath);

function findHome() {
  const candidates = [exeFolder, process.cwd(), resolve(process.cwd(), "..", "..")];
  return candidates.find(folder => existsSync(join(folder, CONFIG_MARKER))) ?? process.cwd();
}

export const home = findHome();
export const configFolder = join(home, "config");

/** A file from `apps/guide/assets`: beside the exe in a release, in the folder from source. */
export function assetFile(name: string) {
  // Beside the exe, beside the bundle (run as `node main.js`), or in the sources.
  const candidates = [join(exeFolder, name), join(home, name), join(home, "apps", "guide", "assets", name)];
  return candidates.find(path => existsSync(path)) ?? "";
}

export const iconFile = () => assetFile("app.ico");

/** The bridge exe: shipped beside the app, or the repository's release build. */
export function bridgeExe() {
  const candidates = [join(home, BRIDGE_EXE), join(home, "target", "release", BRIDGE_EXE)];
  return candidates.find(path => existsSync(path)) ?? null;
}
