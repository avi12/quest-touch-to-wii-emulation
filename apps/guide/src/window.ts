import { runOnUiThread } from "winui-svelte/thread";

// The guide's UI thread: loads the app and runs the window until it closes.

await runOnUiThread(async () => {
  const isFromSource = import.meta.url.endsWith(".ts");
  if (isFromSource) {
    const { registerHooks } = await import("node:module");
    const { svelteHooks } = await import("winui-svelte/testing");
    registerHooks(svelteHooks());
  }

  await import("winui-svelte/components");
  const { mountControl } = await import("winui-svelte");
  const { runWindow } = await import("winui-svelte/window");
  const { default: app } = await import("./App.svelte");
  await runWindow({
    onStart() {
      mountControl({ component: app, props: {} });
    }
  });
});
