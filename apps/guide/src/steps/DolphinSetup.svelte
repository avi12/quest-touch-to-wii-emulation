<script lang="ts">
  import Mapping from "../components/Mapping.svelte";
  import Step from "../components/Step.svelte";
  import { applyDolphinConfig } from "../lib/dolphin/config.ts";
  import { status } from "../lib/status.svelte.ts";
  import { describeError } from "../lib/system/errors.ts";
  import { Button } from "winui-svelte/components";

  let message = $state("");

  async function configure() {
    if (status.isDolphinOpen) {
      message = "Close Dolphin first: it overwrites its config when it exits";
      return;
    }

    try {
      await applyDolphinConfig();
      message = "Done. Your old files were kept as .bak copies";
    } catch (error) {
      message = `Could not write the config: ${describeError(error)}`;
    }

    await status.refresh();
  }
</script>

<Step
  isDone={status.isConfigured}
  number={2}
  status={message || (status.isConfigured ? "Dolphin knows the bridge" : "Dolphin is not set up for the bridge yet")}
  title="Set up Dolphin (once)"
>
  <Mapping
    pairs={[
      ["Right controller", "Wii Remote 1"],
      ["Left controller", "Wii Remote 2"],
      ["Another window focused", "Input keeps working"]
    ]}
  />
  {#snippet actions()}
    <Button isEnabled={!status.isDolphinOpen} onclick={configure}>
      {status.isDolphinOpen ? "Close Dolphin to continue" : "Set up Dolphin"}
    </Button>
  {/snippet}
</Step>
