<script lang="ts">
  import Step from "./Step.svelte";
  import { bridges } from "./lib/bridges.svelte.ts";
  import { applyDolphinConfig, isDolphinConfigured } from "./lib/dolphin-config.ts";
  import { bridgeExe } from "./lib/home.ts";
  import { launchWiiMenu, loadPaths, type Paths, savePaths } from "./lib/settings.ts";
  import { isRunning, openUri, PROCESS, startSteamVr } from "./lib/system.ts";
  import { onMount } from "svelte";
  import type { WindowHandle } from "winui-svelte/components";
  import { Button, ScrollViewer, StackPanel, TextBlock, Window } from "winui-svelte/components";
  import { exitApplication } from "winui-svelte/window";

  // The setup guide: each step says what to do, checks whether it is done, and does what it can.

  const WINDOW_WIDTH = 760;
  const WINDOW_HEIGHT = 920;
  const REFRESH_MS = 2000;
  const README = "https://github.com/avi12/quest-touch-to-wii-emulation#readme";

  let handle: WindowHandle | undefined = $state();
  let isSteamVrUp = $state(false);
  let isDolphinOpen = $state(false);
  let isConfigured = $state(false);
  let configMessage = $state("");
  let paths = $state<Paths>({ dolphin: "", disc: "" });
  const hasBridge = bridgeExe() !== null;

  async function refresh() {
    isSteamVrUp = await isRunning(PROCESS.steamVr);
    isDolphinOpen = await isRunning(PROCESS.dolphin);
    isConfigured = await isDolphinConfigured();
  }

  onMount(() => {
    void loadPaths().then(saved => (paths = saved));
    void refresh();
    const interval = setInterval(() => void refresh(), REFRESH_MS);
    return () => clearInterval(interval);
  });

  async function configure() {
    if (isDolphinOpen) {
      configMessage = "Close Dolphin first: it overwrites its config when it exits";
      return;
    }

    try {
      await applyDolphinConfig();
      configMessage = "Done. Your old files were kept as .bak copies";
    } catch (error) {
      configMessage = `Could not write the config: ${error instanceof Error ? error.message : String(error)}`;
    }

    await refresh();
  }

  async function pick(kind: keyof Paths) {
    const isDolphin = kind === "dolphin";
    const chosen = await handle?.pickFile({
      title: isDolphin ? "Find Dolphin.exe" : "Choose a Wii game",
      fileTypes: isDolphin ? [".exe"] : [".rvz", ".iso", ".wbfs", ".wia", ".gcz"]
    });
    if (!chosen) {
      return;
    }

    paths = { ...paths, [kind]: chosen };
    await savePaths(paths);
  }

  const canLaunch = $derived(paths.dolphin !== "" && paths.disc !== "" && !isDolphinOpen);
  const fileName = (path: string) => path.split("\\").at(-1) ?? path;
</script>

<Window height={WINDOW_HEIGHT} onclosed={exitApplication} title="Touch → Dolphin setup" width={WINDOW_WIDTH} bind:handle>
  <ScrollViewer>
    <StackPanel class="page">
      <TextBlock class="heading">Play Wii games with Quest Touch controllers</TextBlock>
      <TextBlock class="lead">
        Work down the steps. Each one checks itself; the buttons do what can be done for you.
      </TextBlock>

      <Step isDone={isSteamVrUp} number={1} status={isSteamVrUp ? "SteamVR is running" : "SteamVR is not running yet"} title="Connect the headset">
        <TextBlock>
          Put the headset on, open Air Link (Quick Settings → Link) and connect to this PC. Then start SteamVR: it
          only tracks the controllers, nothing is shown in VR.
        </TextBlock>
        {#snippet actions()}
          <Button onclick={startSteamVr}>Start SteamVR</Button>
        {/snippet}
      </Step>

      <Step
        isDone={isConfigured}
        number={2}
        status={configMessage || (isConfigured ? "Dolphin knows the bridge" : "Dolphin is not set up for the bridge yet")}
        title="Set up Dolphin (once)"
      >
        <TextBlock>
          Adds the bridge's two controllers to Dolphin as Wii Remote 1 (right hand) and Wii Remote 2 (left hand), and
          lets the game keep input when another window has focus. Dolphin must be closed.
        </TextBlock>
        {#snippet actions()}
          <Button isEnabled={!isDolphinOpen} onclick={configure}>
            {isDolphinOpen ? "Close Dolphin to continue" : "Set up Dolphin"}
          </Button>
        {/snippet}
      </Step>

      <Step
        isDone={bridges.isRunning}
        number={3}
        status={hasBridge ? bridges.status : "touch_dsu.exe is missing next to this guide"}
        title="Start the bridges"
      >
        <TextBlock>
          Wake both controllers (press a button) and hold them in front of the headset cameras. The bridges turn
          them into Wii Remotes; they update themselves when a new release is out.
        </TextBlock>
        {#each bridges.lines as line, index (index)}
          <TextBlock class="log">{line}</TextBlock>
        {/each}
        {#snippet actions()}
          <Button isEnabled={hasBridge && !bridges.isRunning} onclick={() => bridges.start()}>Start</Button>
          <Button isEnabled={bridges.isRunning} onclick={() => bridges.stop()}>Stop</Button>
        {/snippet}
      </Step>

      <Step
        isDone={isDolphinOpen}
        number={4}
        status={isDolphinOpen
          ? "Dolphin is running"
          : canLaunch
            ? `Ready: ${fileName(paths.disc)}`
            : "Choose Dolphin and a game"}
        title="Play"
      >
        <TextBlock>
          Opens the Wii Menu with your game already in the drive: pick it from the Disc Channel. The first time, the
          Wii Menu needs Dolphin → Tools → Perform Online System Update.
        </TextBlock>
        <TextBlock class="log">Dolphin: {paths.dolphin || "not chosen"}</TextBlock>
        <TextBlock class="log">Game: {paths.disc || "not chosen"}</TextBlock>
        {#snippet actions()}
          <Button onclick={() => pick("dolphin")}>Find Dolphin.exe</Button>
          <Button onclick={() => pick("disc")}>Choose a game</Button>
          <Button isEnabled={canLaunch} onclick={() => launchWiiMenu(paths)}>Open the Wii Menu</Button>
        {/snippet}
      </Step>

      <Step isDone={false} number={5} status="Tip: click the right thumbstick whenever the hand drifts" title="Aim and recenter">
        <TextBlock>
          Aim at the middle of the screen and click the right thumbstick: the hand jumps to the centre. A = A, right
          grip = B, B = 1, Y = 2, X = −, right trigger = Home, left menu = +, right stick = D-pad. Ctrl+Alt+L switches
          the left controller (Nunchuk) on or off.
        </TextBlock>
        {#snippet actions()}
          <Button onclick={() => openUri(README)}>Read the full guide</Button>
        {/snippet}
      </Step>
    </StackPanel>
  </ScrollViewer>
</Window>

<style>
  .page {
    gap: 16px;
    padding: 24px;
  }

  .heading {
    font-size: 26px;
    font-weight: 600;
  }

  .lead {
    opacity: 0.8;
  }

  .log {
    font-family: Consolas;
    font-size: 12px;
    opacity: 0.8;
  }
</style>
