<script lang="ts">
  import Chip from "../components/Chip.svelte";
  import Step from "../components/Step.svelte";
  import { loadPaths, NO_PATHS, type Paths, savePaths } from "../lib/dolphin/launch.ts";
  import { status } from "../lib/status.svelte.ts";
  import { Phase } from "../lib/system/lifecycle.svelte.ts";
  import { basename } from "node:path";
  import { onMount } from "svelte";
  import type { WindowHandle } from "winui-svelte/components";
  import { Button, StackPanel, TextBlock } from "winui-svelte/components";
  import { DropEffect, dropzone } from "winui-svelte/data-transfer";
  import type { DragEvent } from "winui-svelte/data-transfer";

  // Choose Dolphin and a game (with the file picker, or by dropping the game file on this step),
  // then open the Wii Menu with the game in the drive.

  interface Props {
    handle: WindowHandle | undefined;
  }

  const { handle }: Props = $props();
  const GAME_TYPES = [".rvz", ".iso", ".wbfs", ".wia", ".gcz"];
  const DROP_CAPTION = "Use this game";
  const PICKERS: Record<keyof Paths, { title: string; fileTypes: string[] }> = {
    dolphin: { title: "Find Dolphin.exe", fileTypes: [".exe"] },
    disc: { title: "Choose a Wii game", fileTypes: GAME_TYPES }
  };
  const STATUS = {
    stopped: "Choose Dolphin and a game",
    starting: "Opening Dolphin…",
    running: "Dolphin is running",
    stopping: "Dolphin is closing…"
  } as const satisfies Record<Phase, string>;

  let paths = $state<Paths>(NO_PATHS);
  let isDropOver = $state(false);

  onMount(() => {
    void loadPaths().then(saved => (paths = saved));
  });

  async function choose(kind: keyof Paths, path: string) {
    paths = { ...paths, [kind]: path };
    await savePaths(paths);
  }

  async function pick(kind: keyof Paths) {
    const chosen = await handle?.pickFile(PICKERS[kind]);
    if (chosen) {
      await choose(kind, chosen);
    }
  }

  function showDropFeedback(event: DragEvent) {
    event.dataTransfer.dropEffect = DropEffect.link;
    event.setFeedback({ caption: DROP_CAPTION, isGlyphVisible: true });
  }

  async function drop(event: DragEvent) {
    const [game] = event.dataTransfer.files;
    if (game) {
      await choose("disc", game.path);
    }
  }

  const canLaunch = $derived(paths.dolphin !== "" && paths.disc !== "" && !status.isDolphinOpen);
  const label = (path: string) => (path ? basename(path) : "not chosen");
  const statusText = $derived(status.dolphin.phase === Phase.stopped && canLaunch ? "Ready" : STATUS[status.dolphin.phase]);
</script>

<StackPanel
  class="drop"
  class:drop-over={isDropOver}
  {@attach dropzone({
    accept: GAME_TYPES,
    onDragStateChange: isOver => {
      isDropOver = isOver;
    },
    onDragEnter: showDropFeedback,
    onDragOver: showDropFeedback,
    onDrop: event => void drop(event)
  })}
>
  <Step
    isDone={status.dolphin.isRunning}
    number={4}
    phase={status.dolphin.phase}
    status={statusText}
    title="Play"
  >
    <StackPanel class="files">
      {#key paths.dolphin}
        <Chip glyph="OpenFile" label={label(paths.dolphin)} />
      {/key}
      {#key paths.disc}
        <Chip glyph="Game" label={label(paths.disc)} />
      {/key}
    </StackPanel>
    <TextBlock class="note">
      Drop a game file here, or choose one. Opens the Wii Menu with the game in the drive: pick it from the Disc
      Channel. First time only: Dolphin → Tools → Perform Online System Update.
    </TextBlock>
    {#snippet actions()}
      <Button onclick={() => pick("dolphin")}>Find Dolphin.exe</Button>
      <Button onclick={() => pick("disc")}>Choose a game</Button>
      <Button isEnabled={canLaunch} onclick={() => status.openWiiMenu(paths)}>Open the Wii Menu</Button>
    {/snippet}
  </Step>
</StackPanel>

<style>
  .drop {
    border-radius: 8px;
    transition: background-color 150ms;
  }

  .drop-over {
    background-color: var(--accent-fill-color-tertiary);
  }

  .files {
    flex-direction: row;
    gap: 8px;
  }

  .note {
    opacity: 0.8;
  }
</style>
