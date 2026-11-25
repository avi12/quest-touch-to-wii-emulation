import type { FluentGlyphName } from "winui-svelte/components";

/** What a chip shows: a label, with an icon when one says it faster. */
interface ChipItem {
  label: string;
  glyph?: FluentGlyphName;
}

/** A plain label is a chip without an icon. */
export type ChipInput = ChipItem | string;

export const toChip = (input: ChipInput): ChipItem => (typeof input === "string" ? { label: input } : input);
