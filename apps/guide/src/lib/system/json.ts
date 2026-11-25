import { readFile } from "node:fs/promises";

import type { z } from "zod";

/** A JSON file checked against `schema`; null when it is missing, unreadable or does not match. */
export async function readJson<Schema extends z.ZodType>(file: string, schema: Schema) {
  try {
    const parsed = schema.safeParse(JSON.parse(await readFile(file, "utf8")));
    return parsed.success ? parsed.data : null;
  } catch {
    return null;
  }
}
