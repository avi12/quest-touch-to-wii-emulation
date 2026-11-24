import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";

// Questions about the PC: which programs run. Programs are only ever looked up by name here,
// never stopped by name (the user has other copies running).

const run = promisify(execFile);
const STEAMVR_URI = "steam://rungameid/250820";

export const PROCESS = {
  steamVr: "vrserver.exe",
  dolphin: "Dolphin.exe",
  bridge: "touch_dsu.exe"
} as const;

/** Whether a process with this image name is running. */
export async function isRunning(imageName: string) {
  try {
    const { stdout } = await run("tasklist", ["/FI", `IMAGENAME eq ${imageName}`, "/NH"], { windowsHide: true });
    return stdout.toLowerCase().includes(imageName.toLowerCase());
  } catch {
    return false;
  }
}

/** Opens a URI (steam://, https://) the way Explorer would. */
export function openUri(uri: string) {
  spawn("cmd", ["/c", "start", "", uri], { windowsHide: true, detached: true, stdio: "ignore" }).unref();
}

export function startSteamVr() {
  openUri(STEAMVR_URI);
}
