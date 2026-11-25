<script lang="ts">
  import type { FluentGlyphName } from "winui-svelte/components";
  import { Border, FontIcon, StackPanel, TextBlock } from "winui-svelte/components";

  // A labelled pill: a button name, a program, a file. It fades in when it appears (a new
  // label re-creates it), with the `appear` keyframes from Step.svelte.

  interface Props {
    label: string;
    glyph?: FluentGlyphName;
  }

  const { label, glyph }: Props = $props();
</script>

<Border class="chip">
  <StackPanel class="row">
    {#if glyph}
      <FontIcon class="icon" {glyph} />
    {/if}
    <TextBlock>{label}</TextBlock>
  </StackPanel>
</Border>

<style>
  .chip {
    padding: 4px 10px;
    border-radius: 12px;
    background-color: var(--control-fill-color-secondary);
    animation: appear 220ms ease-out;
  }

  @media (prefers-reduced-motion: reduce) {
    .chip {
      animation: none;
    }
  }

  .row {
    flex-direction: row;
    gap: 6px;
  }

  .icon {
    font-size: 14px;
  }
</style>
