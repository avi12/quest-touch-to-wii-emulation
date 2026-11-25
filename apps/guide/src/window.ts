import { type MessagePort, workerData } from "node:worker_threads";

import type { Component } from "svelte";
import { runOnUiThread } from "winui-svelte/thread";

// The app's UI thread: loads the app and runs the window until it closes. From source the app
// and the package's components come through Vite's module runner (main.ts hands over its port),
// so a saved file updates the open window.

interface DevData {
  port: MessagePort;
}

interface Modules {
  mountControl: typeof import("winui-svelte").mountControl;
  runWindow: typeof import("winui-svelte/window").runWindow;
  app: Component;
}

const devPort = (workerData as DevData | null)?.port;

async function loadBuilt(): Promise<Modules> {
  await import("winui-svelte/components");
  const { mountControl } = await import("winui-svelte");
  const { runWindow } = await import("winui-svelte/window");
  const { default: app } = await import("./App.svelte");
  return { mountControl, runWindow, app };
}

async function loadFromSource(port: MessagePort): Promise<Modules> {
  const { createWindowModuleRunner } = await import("winui-svelte/runner");
  const { runner } = createWindowModuleRunner(port);
  await runner.import("winui-svelte/components");
  const { mountControl }: typeof import("winui-svelte") = await runner.import("winui-svelte");
  const { runWindow }: typeof import("winui-svelte/window") = await runner.import("winui-svelte/window");
  const { default: app }: { default: Component } = await runner.import("/src/App.svelte");
  return { mountControl, runWindow, app };
}

await runOnUiThread(async () => {
  const { mountControl, runWindow, app } = devPort ? await loadFromSource(devPort) : await loadBuilt();
  await runWindow({
    onStart() {
      mountControl({ component: app, props: {} });
    }
  });
});
