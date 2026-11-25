<script lang="ts">
  import Flow from "../components/Flow.svelte";
  import Step from "../components/Step.svelte";
  import { status } from "../lib/status.svelte.ts";
  import type { Phase } from "../lib/system/lifecycle.svelte.ts";
  import { Button, TextBlock } from "winui-svelte/components";

  const STATUS = {
    stopped: "SteamVR is not running yet",
    starting: "Starting SteamVR…",
    running: "SteamVR is running",
    stopping: "SteamVR is closing…"
  } as const satisfies Record<Phase, string>;
</script>

{#snippet start()}
  <Button isEnabled={!status.steamVr.isBusy} onclick={() => status.startSteamVr()}>Start SteamVR</Button>
{/snippet}

<!-- The button is only offered while SteamVR is not running yet. -->
<Step
  actions={status.steamVr.isRunning ? undefined : start}
  isDone={status.steamVr.isRunning}
  phase={status.steamVr.phase}
  number={1}
  status={STATUS[status.steamVr.phase]}
  title="Connect the headset"
>
  <Flow
    items={[
      { glyph: "Headset", label: "Headset on" },
      { glyph: "Wifi", label: "Quick Settings → Link" },
      { glyph: "Play", label: "SteamVR" }
    ]}
  />
  <TextBlock class="note">SteamVR only tracks the controllers; nothing is shown in VR.</TextBlock>
</Step>

<style>
  .note {
    opacity: 0.8;
  }
</style>
