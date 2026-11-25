<script lang="ts">
  import { bridges } from "./lib/bridges/bridges.svelte.ts";
  import { status } from "./lib/status.svelte.ts";
  import { iconFile } from "./lib/system/home.ts";
  import Bridges from "./steps/Bridges.svelte";
  import Controls from "./steps/Controls.svelte";
  import DolphinSetup from "./steps/DolphinSetup.svelte";
  import Headset from "./steps/Headset.svelte";
  import Play from "./steps/Play.svelte";
  import Troubleshooting from "./steps/Troubleshooting.svelte";
  import UpdateBar from "./steps/UpdateBar.svelte";
  import { onMount } from "svelte";
  import type { WindowHandle } from "winui-svelte/components";
  import { Button, ScrollViewer, StackPanel, TextBlock, Window } from "winui-svelte/components";
  import { hideToTray, restoreFromTray, toggleInTray, TrayIcon } from "winui-svelte/tray";
  import { exitApplication } from "winui-svelte/window";

  // The window and tray around the steps. While the bridges run, closing the window keeps them
  // going in the tray; Quit stops everything.

  const WINDOW_WIDTH = 600;
  const WINDOW_HEIGHT = 920;
  const TRAY_GUID = "3b6f0d8e-5a41-4c2b-9e7d-1f6a2c8b9d04";

  let handle: WindowHandle | undefined = $state();

  onMount(() => status.watch());

  /** The window's close button: keep playing in the tray while the bridges run. */
  function onclosing() {
    if (!bridges.isAlive || !handle) {
      return false;
    }

    hideToTray(handle);
    return true;
  }

  function quit() {
    bridges.stop();
    exitApplication();
  }
</script>

<Window
  backdrop="acrylic"
  height={WINDOW_HEIGHT}
  icon={iconFile()}
  isMaximizable={false}
  maxWidth={WINDOW_WIDTH}
  minWidth={WINDOW_WIDTH}
  {onclosing}
  onclosed={quit}
  title="Touch → Dolphin"
  width={WINDOW_WIDTH}
  bind:handle
>
  <ScrollViewer>
    <StackPanel class="page">
      <TextBlock class="heading">Play Wii games with Quest Touch controllers</TextBlock>
      <UpdateBar onInstalled={quit} />
      <Headset />
      <DolphinSetup />
      <Bridges />
      <Play {handle} />
      <Controls />
      <Troubleshooting />
      <TextBlock class="lead">Closing the window keeps the bridges running in the tray.</TextBlock>
      <Button onclick={quit}>Quit (stops the bridges)</Button>
    </StackPanel>
  </ScrollViewer>
</Window>
<TrayIcon
  guid={TRAY_GUID}
  icon={iconFile()}
  onclick={() => handle && toggleInTray(handle)}
  oncontextmenu={() => handle && restoreFromTray(handle)}
  tooltip={`Touch → Dolphin: bridges ${bridges.lifecycle.phase}`}
/>

<style>
  .page {
    gap: 16px;
    padding: 24px;
  }

  .heading {
    font-size: 24px;
    font-weight: 600;
  }

  .lead {
    opacity: 0.8;
  }
</style>
