<script lang="ts">
  import { findUpdate, installUpdate } from "../lib/app/update.ts";
  import { describeError } from "../lib/system/errors.ts";
  import { onMount } from "svelte";
  import { Button, InfoBar } from "winui-svelte/components";

  // Offers the newest release when this build is older. `onInstalled` quits the app.

  interface Props {
    onInstalled: () => void;
  }

  const { onInstalled }: Props = $props();
  let update = $state<string | null>(null);
  let message = $state("");

  onMount(() => {
    void findUpdate().then(found => (update = found));
  });

  async function apply() {
    if (!update) {
      return;
    }

    message = `Downloading ${update}…`;
    try {
      await installUpdate(update);
      onInstalled();
    } catch (error) {
      message = `Update failed: ${describeError(error)}`;
    }
  }
</script>

{#if update}
  <InfoBar
    isClosable={false}
    isOpen
    message={message || `Version ${update} is out. The app restarts on its own after updating.`}
    title="Update available"
  >
    {#snippet actionButton()}
      <Button onclick={apply}>Update now</Button>
    {/snippet}
  </InfoBar>
{/if}
