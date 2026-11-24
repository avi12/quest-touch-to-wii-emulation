import { spawn } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";

// The Dolphin and game paths the person picked, kept between runs.

const SETTINGS_FILE = join(process.env["APPDATA"] ?? "", "touch_dsu", "guide.json");
const WII_MENU_TITLE = "0000000100000002";

export interface Paths {
  dolphin: string;
  disc: string;
}

export async function loadPaths(): Promise<Paths> {
  try {
    const saved: unknown = JSON.parse(await readFile(SETTINGS_FILE, "utf8"));
    const record = typeof saved === "object" && saved !== null ? (saved as Record<string, unknown>) : {};
    return {
      dolphin: typeof record["dolphin"] === "string" ? record["dolphin"] : "",
      disc: typeof record["disc"] === "string" ? record["disc"] : ""
    };
  } catch {
    return { dolphin: "", disc: "" };
  }
}

export async function savePaths(paths: Paths) {
  await mkdir(dirname(SETTINGS_FILE), { recursive: true });
  await writeFile(SETTINGS_FILE, `${JSON.stringify(paths, null, 2)}\n`);
}

/** Dolphin's Wii Menu with the game already in the drive (Disc Channel). */
export function launchWiiMenu({ dolphin, disc }: Paths) {
  spawn(dolphin, ["-C", `Dolphin.Core.DefaultISO=${disc}`, "-n", WII_MENU_TITLE], {
    detached: true,
    stdio: "ignore"
  }).unref();
}
