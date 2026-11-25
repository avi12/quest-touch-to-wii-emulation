<script lang="ts">
  import Flow from "./Flow.svelte";
  import type { ChipInput } from "./chip.ts";
  import { StackPanel } from "winui-svelte/components";

  // A table of "this becomes that", one arrow per row.

  interface Props {
    pairs: readonly (readonly [from: ChipInput, to: ChipInput])[];
  }

  const { pairs }: Props = $props();
</script>

<StackPanel class="rows">
  {#each pairs as pair, index (index)}
    <Flow items={pair} />
  {/each}
</StackPanel>

<style>
  .rows {
    gap: 6px;
  }
</style>
