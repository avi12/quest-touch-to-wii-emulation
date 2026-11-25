<script lang="ts">
  import Chip from "./Chip.svelte";
  import { type ChipInput, toChip } from "./chip.ts";
  import { FontIcon, StackPanel } from "winui-svelte/components";

  // Chips in order, joined by arrows: do this, then this.

  interface Props {
    items: readonly ChipInput[];
  }

  const { items }: Props = $props();
</script>

<StackPanel class="flow">
  {#each items as item, index (index)}
    {@const chip = toChip(item)}
    {#if index > 0}
      <FontIcon class="arrow" glyph="ChevronRight" />
    {/if}
    <Chip glyph={chip.glyph} label={chip.label} />
  {/each}
</StackPanel>

<style>
  .flow {
    flex-direction: row;
    gap: 6px;
  }

  .arrow {
    font-size: 12px;
    opacity: 0.6;
  }
</style>
