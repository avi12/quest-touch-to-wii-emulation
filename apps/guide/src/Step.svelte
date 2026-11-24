<script lang="ts">
  import type { Snippet } from "svelte";
  import { Border, InfoBar, StackPanel, TextBlock } from "winui-svelte/components";
  import { InfoBarSeverity } from "winui-svelte/enums";

  // One numbered step: what to do, whether it is done, and the buttons that do it.

  interface Props {
    number: number;
    title: string;
    isDone: boolean;
    status: string;
    children: Snippet;
    actions?: Snippet;
  }

  const { number, title, isDone, status, children, actions }: Props = $props();
</script>

<Border class="card">
  <StackPanel class="body">
    <TextBlock class="title">{number}. {title}</TextBlock>
    {@render children()}
    <InfoBar
      isClosable={false}
      isOpen
      message={status}
      severity={isDone ? InfoBarSeverity.Success : InfoBarSeverity.Informational}
    />
    {#if actions}
      <StackPanel class="actions">
        {@render actions()}
      </StackPanel>
    {/if}
  </StackPanel>
</Border>

<style>
  .card {
    padding: 16px;
    border-radius: 8px;
    background: var(--CardBackgroundFillColorDefaultBrush);
  }

  .body {
    gap: 8px;
  }

  .title {
    font-size: 18px;
    font-weight: 600;
  }

  .actions {
    flex-direction: row;
    gap: 8px;
  }
</style>
