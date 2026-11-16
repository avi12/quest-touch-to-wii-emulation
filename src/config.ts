// Runtime settings, read once from the environment and validated, so a typo fails at start-up instead of mid-game.
import { z } from "zod";

const envSchema = z.object({
  BRIDGE_PORT: z.coerce.number().int().min(1).max(65535).default(26760),
  BRIDGE_HAND: z.enum(["left", "right"]).default("right"), // "left": the second instance, the LEFT controller is Wii Remote 2
  PTR_TEST: z.string().optional(), // diagnostic: sweep known pointer values, ignoring the controller
  BRIDGE_TELEMETRY: z.string().optional() // file to stream controller movement, pointer and sensors to (JSON lines)
});

const env = envSchema.parse(process.env);

export const config = {
  port: env.BRIDGE_PORT,
  hand: env.BRIDGE_HAND,
  isPlayer2: env.BRIDGE_HAND === "left",
  isPointerTest: Boolean(env.PTR_TEST),
  telemetryPath: env.BRIDGE_TELEMETRY
} as const;
