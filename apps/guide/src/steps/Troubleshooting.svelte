<script lang="ts">
  import Mapping from "../components/Mapping.svelte";
  import Step from "../components/Step.svelte";
  import { formatDistanceRange } from "../lib/system/units.ts";

  // The README's troubleshooting table, the rows a player can act on. Keep in step with it.

  const FIXES = [
    ["No hand on screen", "Aim at the screen, click the right stick"],
    ["Controllers vanish", "Press a button; keep them in the headset's view"],
    ["Pointer drifts", `Headset on a stand facing you, ${formatDistanceRange(1, 2)} away`],
    ["Buttons do nothing", "Close the headset menu, or restart SteamVR"],
    ["Game froze after a key", "F10 is Dolphin's Pause: press it again"],
    ["Headset restarted", "Battery ran out: charge it, reconnect"]
  ] as const;
</script>

<Step isDone={false} number={6} status="Still stuck? The README on GitHub has the full table" title="If something is off">
  <Mapping pairs={FIXES} />
</Step>
