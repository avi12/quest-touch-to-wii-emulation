import { existsSync } from "node:fs";
import { copyFile, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

import { configFolder } from "../system/home.ts";

// Merges the bridge's Dolphin config into Dolphin's own. Dolphin rewrites its config on exit, so
// this runs only while Dolphin is closed; every file it changes is backed up first.

export const dolphinConfigFolder = join(process.env["APPDATA"] ?? "", "Dolphin Emulator", "Config");
const WIIMOTE_SECTIONS = ["Wiimote1", "Wiimote2"];

type Sections = Map<string, string[]>;

/** INI text as section name -> lines (the lines before the first header go under ""). */
function parse(text: string): Sections {
  const sections: Sections = new Map([["", []]]);
  let current = "";
  for (const line of text.split(/\r?\n/)) {
    const header = /^\[(.+)\]\s*$/.exec(line);
    if (header?.[1]) {
      current = header[1];
      sections.set(current, []);
      continue;
    }

    sections.get(current)?.push(line);
  }

  return sections;
}

function render(sections: Sections) {
  const parts = [...sections].flatMap(([name, lines]) => {
    const body = lines.filter((line, index) => line !== "" || index < lines.length - 1);
    return name === "" ? body : [`[${name}]`, ...body];
  });
  return `${parts.join("\n").trim()}\n`;
}

async function backUp(file: string) {
  if (existsSync(file)) {
    await copyFile(file, `${file}.bak-${Date.now()}`);
  }
}

async function readSections(file: string) {
  return existsSync(file) ? parse(await readFile(file, "utf8")) : parse("");
}

/** Copies DSUClient.ini, merges Wii Remote 1 and 2, and turns on background input. */
export async function applyDolphinConfig() {
  const dsuTarget = join(dolphinConfigFolder, "DSUClient.ini");
  await backUp(dsuTarget);
  await copyFile(join(configFolder, "DSUClient.ini"), dsuTarget);

  const wiimoteTarget = join(dolphinConfigFolder, "WiimoteNew.ini");
  const ours = await readSections(join(configFolder, "WiimoteNew.ini"));
  const theirs = await readSections(wiimoteTarget);
  for (const name of WIIMOTE_SECTIONS) {
    theirs.set(name, ours.get(name) ?? []);
  }

  await backUp(wiimoteTarget);
  await writeFile(wiimoteTarget, render(theirs));

  const mainTarget = join(dolphinConfigFolder, "Dolphin.ini");
  const main = await readSections(mainTarget);
  const input = (main.get("Input") ?? []).filter(line => !line.startsWith("BackgroundInput"));
  main.set("Input", ["BackgroundInput = True", ...input]);
  await backUp(mainTarget);
  await writeFile(mainTarget, render(main));
}

/** Whether Dolphin already reads the bridge's two DSU servers. */
export async function isDolphinConfigured() {
  const dsu = join(dolphinConfigFolder, "DSUClient.ini");
  return existsSync(dsu) && (await readFile(dsu, "utf8")).includes("26761");
}
