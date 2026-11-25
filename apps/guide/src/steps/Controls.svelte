<script lang="ts">
  import Mapping from "../components/Mapping.svelte";
  import Step from "../components/Step.svelte";
  import ControllerDiagram from "../components/ControllerDiagram.svelte";
  import { TextBlock } from "winui-svelte/components";

  // The README's controls and hotkeys: each controller drawn with what its buttons do in the game
  // (assets/controllers), then the motions and hotkeys. Keep in step with the README's tables.

  const MOTIONS = [
    ["Aim", "Pointer"],
    ["Roll", "Hand tilts"],
    ["Swing fast", "Motion"]
  ] as const;

  const HOTKEYS = [
    ["Ctrl+Alt+L", "Left controller on / off"],
    ["Ctrl+Alt+[ or ]", "Cycle the aiming axis"]
  ] as const;
</script>

<Step isDone={false} number={5} status="Aim at the middle of the screen, then click the right stick" title="Controls">
  <TextBlock class="label">Right controller = Wii Remote</TextBlock>
  <ControllerDiagram hand="right" />
  <TextBlock class="label">Left controller = Nunchuk</TextBlock>
  <ControllerDiagram hand="left" />
  <TextBlock class="label">Moving the right controller</TextBlock>
  <Mapping pairs={MOTIONS} />
  <TextBlock class="label">Hotkeys (either Ctrl and either Alt)</TextBlock>
  <Mapping pairs={HOTKEYS} />
</Step>

<style>
  .label {
    font-weight: 600;
  }
</style>
