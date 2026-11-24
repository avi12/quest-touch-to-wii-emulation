import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

import { buildApp } from "winui-svelte/vite";

// Packages the guide for a release, into <repo>/target/guide:
//   Setup Guide.exe   Node's single-executable app, switched to the GUI subsystem (no console)
//   main.js, window.js, node_modules, bin, css   what buildApp emits; the exe runs main.js
//   touch_dsu.exe, config/   the bridge and the Dolphin config the guide installs
// The release workflow zips that folder.

const root = import.meta.dirname;
const repo = path.resolve(root, "..", "..");
const output = path.join(repo, "target", "guide");
const exeName = "Setup Guide.exe";
const PE_POINTER_OFFSET = 0x3c;
const SUBSYSTEM_OFFSET_IN_OPTIONAL_HEADER = 68;
const OPTIONAL_HEADER_OFFSET = 24;
const GUI_SUBSYSTEM = 2;

rmSync(output, { recursive: true, force: true });
await buildApp({
  root,
  entries: {
    main: path.join(root, "main.ts"),
    window: path.join(root, "src", "window.ts")
  },
  outDirectory: output
});

// The exe is a stub that loads main.js from beside itself, so the native addon and the
// Windows App SDK files stay ordinary files on disk.
const stub = path.join(output, "sea-stub.cjs");
writeFileSync(
  stub,
  `const path = require("node:path");
const { createRequire } = require("node:module");
// A single-executable app's import() reaches only built-ins; require() loads main.js (ESM, no
// top-level await) from disk beside the exe.
const main = path.join(path.dirname(process.execPath), "main.js");
createRequire(main)(main);
`
);
const seaConfig = path.join(output, "sea-config.json");
writeFileSync(
  seaConfig,
  JSON.stringify({ main: stub, output: path.join(output, exeName), disableExperimentalSEAWarning: true })
);
execFileSync(process.execPath, ["--build-sea", seaConfig], { stdio: "inherit" });
rmSync(stub);
rmSync(seaConfig);

// Node's exe is a console program; flip the PE subsystem so a double-click opens no console.
const exe = path.join(output, exeName);
const bytes = readFileSync(exe);
const peOffset = bytes.readUInt32LE(PE_POINTER_OFFSET);
bytes.writeUInt16LE(GUI_SUBSYSTEM, peOffset + OPTIONAL_HEADER_OFFSET + SUBSYSTEM_OFFSET_IN_OPTIONAL_HEADER);
writeFileSync(exe, bytes);

// main.js and window.js are ES modules; say so, so Node does not parse them twice.
writeFileSync(path.join(output, "package.json"), JSON.stringify({ type: "module" }));
cpSync(path.join(repo, "target", "release", "touch_dsu.exe"), path.join(output, "touch_dsu.exe"));
mkdirSync(path.join(output, "config"), { recursive: true });
cpSync(path.join(repo, "config"), path.join(output, "config"), { recursive: true });
process.stdout.write(`guide packaged in ${output}\n`);
