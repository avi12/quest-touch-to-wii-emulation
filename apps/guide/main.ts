import { existsSync } from "node:fs";
import path from "node:path";
import { Worker } from "node:worker_threads";

import { initWindowsAppSdk } from "winui-svelte/bootstrap";

// The guide's host: starts the Windows App SDK and runs the window on its own UI thread.
// Built (dist-app/main.js) it loads the built window.js beside it; from source, src/window.ts.

initWindowsAppSdk();
const built = path.join(import.meta.dirname, "window.js");
const worker = new Worker(existsSync(built) ? built : path.join(import.meta.dirname, "src", "window.ts"));
worker.on("error", error => {
  process.stderr.write(`guide: ${error.message}\n`);
  process.exitCode = 1;
});
