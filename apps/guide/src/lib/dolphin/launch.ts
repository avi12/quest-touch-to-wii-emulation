import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";

import { z } from "zod";

import { readJson } from "../system/json.ts";

// The Dolphin and game paths the person picked, kept between runs.

const SETTINGS_FILE = join(process.env["APPDATA"] ?? "", "touch_dsu", "guide.json");
const WII_MENU_TITLE = "0000000100000002";

const PathsSchema = z.object({
  dolphin: z.string().catch(""),
  disc: z.string().catch("")
});

export type Paths = z.infer<typeof PathsSchema>;

export const NO_PATHS: Paths = { dolphin: "", disc: "" };

export async function loadPaths() {
  return (await readJson(SETTINGS_FILE, PathsSchema)) ?? NO_PATHS;
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
