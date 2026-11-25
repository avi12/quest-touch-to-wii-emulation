import { execFileSync } from "node:child_process";
import { copyFile, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";

import { z } from "zod";

import { assetFile } from "../system/home.ts";
import { readJson } from "../system/json.ts";
import { REPOSITORY } from "./repository.ts";

// The app updates itself from the newest GitHub release, like the bridge does: the release's zip
// is downloaded, and once this process has exited assets/install.ps1 copies it over the app
// folder and starts the app again. Only the packaged app updates; from source there is nothing
// to replace.

const ASSET = "touch-dolphin.zip";
const APP_EXE = "Touch Dolphin.exe";
const TAG_PATH = "/releases/tag/v";

type Version = [number, number, number];

const VersionSchema = z
  .string()
  .regex(/^\d+\.\d+\.\d+$/)
  .transform((text): Version => {
    const [major = 0, minor = 0, patch = 0] = text.split(".").map(Number);
    return [major, minor, patch];
  });

const ManifestSchema = z.object({ version: VersionSchema });

function isNewer(candidate: Version, current: Version) {
  const index = candidate.findIndex((part, position) => part !== current[position]);
  return index !== -1 && (candidate[index] ?? 0) > (current[index] ?? 0);
}

const appFolder = dirname(process.execPath);
const isPackaged = basename(process.execPath) === APP_EXE;

/** The newest release's version, from where releases/latest redirects (no API, no token). */
async function latestVersion() {
  const response = await fetch(`${REPOSITORY}/releases/latest`, { redirect: "manual" });
  const location = response.headers.get("location") ?? "";
  const index = location.lastIndexOf(TAG_PATH);
  return index === -1 ? null : (VersionSchema.safeParse(location.slice(index + TAG_PATH.length)).data ?? null);
}

/** The newer version's text when an update is available, else null. Never throws. */
export async function findUpdate() {
  if (!isPackaged) {
    return null;
  }

  try {
    // The version build.ts wrote beside the exe.
    const manifest = await readJson(join(appFolder, "package.json"), ManifestSchema);
    const latest = manifest && (await latestVersion());
    return latest && isNewer(latest, manifest.version) ? latest.join(".") : null;
  } catch {
    return null;
  }
}

/**
 * Downloads `version` and hands over to install.ps1, which waits for this process to exit,
 * copies the files in and restarts the app. The caller stops the bridges and exits.
 */
export async function installUpdate(version: string) {
  const response = await fetch(`${REPOSITORY}/releases/download/v${version}/${ASSET}`);
  if (!response.ok) {
    throw new Error(`download answered HTTP ${response.status}`);
  }

  const work = await mkdtemp(join(tmpdir(), "touch-dolphin-update-"));
  const zip = join(work, ASSET);
  await writeFile(zip, Buffer.from(await response.arrayBuffer()));
  // A copy, so the script never overwrites itself while copying the new release in.
  const script = join(work, "install.ps1");
  await copyFile(assetFile("install.ps1"), script);
  // Started through WMI so it is not in this app's job object: a plain child would be ended
  // together with the app, before it could copy anything.
  const command = [
    "powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden",
    `-File "${script}" -AppPid ${process.pid} -Zip "${zip}" -Target "${appFolder}" -Exe "${APP_EXE}"`
  ].join(" ");
  execFileSync(
    "powershell",
    [
      "-NoProfile",
      "-Command",
      `Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = '${command.replaceAll("'", "''")}' } | Out-Null`
    ],
    { windowsHide: true }
  );
}
