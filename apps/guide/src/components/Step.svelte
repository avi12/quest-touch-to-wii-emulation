<script lang="ts">
  import { Phase } from "../lib/system/lifecycle.svelte.ts";
  import type { Snippet } from "svelte";
  import { Border, InfoBar, ProgressRing, StackPanel, TextBlock } from "winui-svelte/components";
  import { InfoBarSeverity } from "winui-svelte/enums";

  // One numbered step: what to do, whether it is done, and the buttons that do it. In CSS: the
  // cards slide in one after another when the window opens, a spinner fades in while what the
  // step starts or stops is on its way (`phase` starting or stopping), and each new status fades
  // in. Reduced motion turns all of it off.

  interface Props {
    number: number;
    title: string;
    isDone: boolean;
    status: string;
    phase?: Phase;
    children: Snippet;
    actions?: Snippet;
  }

  const { number, title, isDone, status, phase, children, actions }: Props = $props();
  const STAGGER_MS = 70;
  const isBusy = $derived(phase === Phase.starting || phase === Phase.stopping);
</script>

<Border class="card" style:animation-delay="{number * STAGGER_MS}ms">
  <StackPanel class="body">
    <StackPanel class="header">
      <TextBlock class="title">{number}. {title}</TextBlock>
      <ProgressRing class="spinner" class:busy={isBusy} isActive={isBusy} />
    </StackPanel>
    {@render children()}
    {#key status}
      <InfoBar
        class="appear"
        isClosable={false}
        isOpen
        message={status}
        severity={isDone ? InfoBarSeverity.Success : InfoBarSeverity.Informational}
      />
    {/key}
    {#if actions}
      <StackPanel class="actions">
        {@render actions()}
      </StackPanel>
    {/if}
  </StackPanel>
</Border>

<style>
  /* Global: Chip fades in with it too. */
  @keyframes -global-appear {
    from {
      opacity: 0;
    }

    to {
      opacity: 1;
    }
  }

  @keyframes enter {
    from {
      opacity: 0;
      translate: 0 16px;
    }

    to {
      opacity: 1;
      translate: 0 0;
    }
  }

  .card {
    padding: 16px;
    border-radius: 8px;
    background-color: var(--card-background-fill-color-default);
    animation: enter 320ms cubic-bezier(0.16, 1, 0.3, 1) backwards;
  }

  .appear {
    animation: appear 220ms ease-out;
  }

  .body {
    gap: 8px;
  }

  .header {
    flex-direction: row;
    gap: 10px;
  }

  .title {
    font-size: 18px;
    font-weight: 600;
  }

  .spinner {
    width: 18px;
    height: 18px;
    opacity: 0;
    transition: opacity 200ms;
  }

  .busy {
    opacity: 1;
  }

  @media (prefers-reduced-motion: reduce) {
    .card,
    .appear {
      animation: none;
    }

    .spinner {
      transition: none;
    }
  }

  .actions {
    flex-direction: row;
    gap: 8px;
  }
</style>
