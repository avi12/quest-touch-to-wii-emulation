import { existsSync } from "node:fs";
import path from "node:path";
import { MessageChannel, Worker } from "node:worker_threads";

import { initWindowsAppSdk } from "winui-svelte/bootstrap";

// The app's host: starts the Windows App SDK and runs the window on its own UI thread. Built
// (dist-app/main.js) it loads the built window.js beside it. From source it also starts Vite's
// module server, so a saved file updates the open window without a restart.

initWindowsAppSdk();
const built = path.join(import.meta.dirname, "window.js");
const isBuilt = existsSync(built);

async function startWorker() {
  if (isBuilt) {
    return new Worker(built);
  }

  const { startModuleServer } = await import("winui-svelte/vite");
  const { port1, port2 } = new MessageChannel();
  await startModuleServer({
    root: import.meta.dirname,
    port: port1,
    configFile: path.join(import.meta.dirname, "vite.config.ts")
  });
  return new Worker(path.join(import.meta.dirname, "src", "window.ts"), {
    workerData: { port: port2 },
    transferList: [port2]
  });
}

// No top-level await: the single executable's stub loads this file with require().
void startWorker().then(worker => {
  worker.on("error", error => {
    process.stderr.write(`app: ${error.message}\n`);
    process.exitCode = 1;
  });
  worker.on("exit", () => process.exit());
});
