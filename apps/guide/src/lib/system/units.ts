// Distances in the units of the person's region: feet where the region measures in them (the US,
// Liberia, Myanmar), metres elsewhere.

const IMPERIAL_REGIONS = new Set(["US", "LR", "MM"]);
const FEET_PER_METRE = 3.28084;

const locale = new Intl.Locale(Intl.DateTimeFormat().resolvedOptions().locale).maximize();
const isImperial = IMPERIAL_REGIONS.has(locale.region ?? "");

/** A distance range given in metres, e.g. "1–2 m" or "3–7 ft", rounded to whole units. */
export function formatDistanceRange(fromMetres: number, toMetres: number) {
  const unit = isImperial ? "foot" : "meter";
  const factor = isImperial ? FEET_PER_METRE : 1;
  const format = new Intl.NumberFormat(locale.baseName, { style: "unit", unit, maximumFractionDigits: 0 });
  return format.formatRange(Math.round(fromMetres * factor), Math.round(toMetres * factor));
}
