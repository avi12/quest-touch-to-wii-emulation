<script lang="ts">
  import Flow from "../components/Flow.svelte";
  import Step from "../components/Step.svelte";
  import { bridges } from "../lib/bridges/bridges.svelte.ts";
  import { status } from "../lib/status.svelte.ts";
  import { Button, TextBlock } from "winui-svelte/components";
</script>

<Step
  isDone={bridges.lifecycle.isRunning}
  phase={bridges.lifecycle.phase}
  number={3}
  status={status.hasBridge ? bridges.status : "touch_dsu.exe is missing next to this app"}
  title="Start the bridges"
>
  <Flow
    items={[
      { glyph: "Remote", label: "Press a button" },
      { glyph: "Video", label: "Hold in view" },
      { glyph: "Game", label: "Wii Remotes" }
    ]}
  />
  {#each bridges.lines as line, index (index)}
    <TextBlock class="log">{line}</TextBlock>
  {/each}
  {#snippet actions()}
    <Button isEnabled={status.hasBridge && !bridges.isAlive} onclick={() => bridges.start()}>Start</Button>
    <Button isEnabled={bridges.lifecycle.isRunning} onclick={() => bridges.stop()}>Stop</Button>
  {/snippet}
</Step>

<style>
  .log {
    font-family: Consolas;
    font-size: 12px;
    opacity: 0.8;
  }
</style>
