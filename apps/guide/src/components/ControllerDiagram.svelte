<script lang="ts">
  import { assetFile } from "../lib/system/home.ts";
  import { readFileSync } from "node:fs";
  import { z } from "zod";
  import { Border, Canvas, Image, TextBlock } from "winui-svelte/components";

  // A Touch controller beside a Wii Remote, the same numbered badge on a Touch button and the Wii
  // button it becomes, and labelled lines for parts no Wii Remote view shows. tools/draw_controllers.py
  // writes the SVG and where the badges and labels go (WinUI's SVG renderer draws no text).

  interface Props {
    hand: "left" | "right";
  }

  const { hand }: Props = $props();

  const LabelSchema = z.object({ text: z.string(), y: z.number(), color: z.string() });
  // The same number beside a Touch button and the Wii button it becomes.
  const BadgeSchema = z.object({ number: z.number(), color: z.string(), x: z.number(), y: z.number() });
  const HandSchema = z.object({ height: z.number(), labels: z.array(LabelSchema), badges: z.array(BadgeSchema) });
  const CalloutsSchema = z.object({
    width: z.number(),
    labelWidth: z.number(),
    hands: z.object({ left: HandSchema, right: HandSchema })
  });
  const LABEL_HEIGHT = 26;
  const BADGE_SIZE = 18;

  // Read fresh on every load, so a regenerated drawing shows without a restart.
  const callouts = CalloutsSchema.parse(JSON.parse(readFileSync(assetFile("controllers/callouts.json"), "utf8")));
  const drawing = $derived(readFileSync(assetFile(`controllers/touch-${hand}.svg`)));
  const { height, labels, badges } = $derived(callouts.hands[hand]);
</script>

<Canvas {height} width={callouts.width}>
  <Image {height} source={drawing} width={callouts.width} />
  {#each badges as badge (`${badge.number}-${badge.x}`)}
    <Border
      class="badge"
      background={badge.color}
      canvasLeft={badge.x - BADGE_SIZE / 2}
      canvasTop={badge.y - BADGE_SIZE / 2}
    >
      <TextBlock class="number">{badge.number}</TextBlock>
    </Border>
  {/each}
  {#each labels as label (label.text)}
    <Border canvasLeft={0} canvasTop={label.y - LABEL_HEIGHT / 2} width={callouts.labelWidth}>
      <Border class="pill" background={label.color} horizontalAlignment="Right">
        <TextBlock class="text">{label.text}</TextBlock>
      </Border>
    </Border>
  {/each}
</Canvas>

<style>
  .pill {
    height: 26px;
    padding: 0 10px;
    border-radius: 13px;
  }

  .badge {
    width: 18px;
    height: 18px;
    border-radius: 9px;
    border: 1.5px solid white;
  }

  .number {
    align-self: center;
    justify-self: center;
    color: white;
    font-size: 11px;
    font-weight: 700;
  }

  .text {
    align-self: center;
    color: white;
    font-size: 12px;
    font-weight: 600;
  }
</style>
