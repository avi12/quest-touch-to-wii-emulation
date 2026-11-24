import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";

// The app updates itself from the newest GitHub release, like the bridge does: the release's
// zip is downloaded and unpacked, and once this process has exited a small PowerShell script
// copies it over the app folder and starts the app again. Only the packaged app updates; run
// from source, there is nothing to replace.

const REPOSITORY = "https://github.com/avi12/quest-touch-to-wii-emulation";
const ASSET = "touch-dolphin.zip";
const APP_EXE = "Touch Dolphin.exe";
const TAG_PATH = "/releases/tag/v";

type Version = [number, number, number];

function parseVersion(text: string): Version | null {
  const parts = text.split(".").map(Number);
  const isValid = parts.length === 3 && parts.every(Number.isInteger);
  return isValid ? [parts[0] ?? 0, parts[1] ?? 0, parts[2] ?? 0] : null;
}

function isNewer(candidate: Version, current: Version) {
  const index = candidate.findIndex((part, position) => part !== current[position]);
  return index !== -1 && (candidate[index] ?? 0) > (current[index] ?? 0);
}

const appFolder = dirname(process.execPath);
const isPackaged = basename(process.execPath) === APP_EXE;

/** This build's version, written beside the exe by build.ts. */
function currentVersion() {
  const manifest = join(appFolder, "package.json");
  if (!existsSync(manifest)) {
    return null;
  }

  const parsed: unknown = JSON.parse(readFileSync(manifest, "utf8"));
  const version = typeof parsed === "object" && parsed !== null ? (parsed as Record<string, unknown>)["version"] : null;
  return typeof version === "string" ? parseVersion(version) : null;
}

/** The newest release's version, from where releases/latest redirects (no API, no token). */
async function latestVersion() {
  const response = await fetch(`${REPOSITORY}/releases/latest`, { redirect: "manual" });
  const location = response.headers.get("location") ?? "";
  const index = location.lastIndexOf(TAG_PATH);
  return index === -1 ? null : parseVersion(location.slice(index + TAG_PATH.length));
}

/** The newer version's text when an update is available, else null. Never throws. */
export async function findUpdate() {
  try {
    const current = currentVersion();
    if (!isPackaged || !current) {
      return null;
    }

    const latest = await latestVersion();
    return latest && isNewer(latest, current) ? latest.join(".") : null;
  } catch {
    return null;
  }
}

/**
 * Downloads and unpacks `version`, then hands over to a script that waits for this process to
 * exit, copies the files in and restarts the app. The caller stops the bridges and exits.
 */
export async function installUpdate(version: string) {
  const work = await mkdtemp(join(tmpdir(), "touch-dolphin-update-"));
  const response = await fetch(`${REPOSITORY}/releases/download/v${version}/${ASSET}`);
  if (!response.ok) {
    throw new Error(`download answered HTTP ${response.status}`);
  }

  const zip = join(work, ASSET);
  await writeFile(zip, Buffer.from(await response.arrayBuffer()));
  const script = join(work, "install.ps1");
  await writeFile(
    script,
    [
      "param([int]$AppPid, [string]$Zip, [string]$Target, [string]$Exe)",
      "Wait-Process -Id $AppPid -ErrorAction SilentlyContinue",
      "$unpacked = Join-Path (Split-Path $Zip) 'unpacked'",
      "Expand-Archive -Path $Zip -DestinationPath $unpacked -Force",
      "Copy-Item -Path (Join-Path $unpacked '*') -Destination $Target -Recurse -Force",
      "Start-Process -FilePath (Join-Path $Target $Exe) -WorkingDirectory $Target",
      ""
    ].join("\n")
  );
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
