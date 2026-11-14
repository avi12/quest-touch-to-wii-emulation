// Connect to a running bridge as a DSU client and print what Dolphin would see.
//
//     node tools/dsu_probe.ts [port] [seconds]
//
// Prints the min/max of the pointer axes (128 = centre), the largest gyro rate and the pressed buttons.
// Use it to check the bridge without Dolphin: move the controller while it runs.
import dgram from "node:dgram";

import { z } from "zod";

import { DSU_EVENT, frameDsu } from "#/dsu_server.ts";

const DEFAULT_PORT = 26760;
const DEFAULT_SECONDS = 5;
const MAX_PORT = 65535;
const LOOPBACK = "127.0.0.1";
const CLIENT_ID = 1234;
const REPLY_TIMEOUT_MS = 1000;
const MS_PER_SECOND = 1000;
const REQUEST_BODY_BYTES = 9; // actions, slot, MAC (6 bytes), all zero: "any pad"
const PAD_PACKET_MIN_BYTES = 100;
const OFFSET = {
  buttons1: 36,
  buttons2: 37,
  pointerX: 42,
  pointerY: 43,
  gyro: 88
} as const;
const GYRO_AXES = 3;
const FLOAT_BYTES = 4;
const BYTE_BITS = 8;
const HEX_DIGITS = 4;
const HEX_RADIX = 16;
const EXIT_BAD_USAGE = 2;

const argsSchema = z.tuple([
  z.coerce.number().int().min(1).max(MAX_PORT).default(DEFAULT_PORT),
  z.coerce.number().positive().default(DEFAULT_SECONDS)
]);

function readArgs() {
  const raw = process.argv.slice(2);
  const parsed = argsSchema.safeParse([raw[0], raw[1]]);
  if (parsed.success) {
    return {
      port: parsed.data[0],
      seconds: parsed.data[1]
    };
  }

  console.error(`usage: node tools/dsu_probe.ts [port 1-${MAX_PORT}] [seconds > 0]\n${z.prettifyError(parsed.error)}`);
  return process.exit(EXIT_BAD_USAGE);
}

function nextDatagram({ socket, timeoutMs }: { socket: dgram.Socket; timeoutMs: number }) {
  return new Promise<Buffer | null>(resolve => {
    const timer = setTimeout(() => {
      socket.off("message", onMessage);
      resolve(null);
    }, timeoutMs);
    function onMessage(data: Buffer) {
      clearTimeout(timer);
      resolve(data);
    }
    socket.once("message", onMessage);
  });
}

function peakGyro(data: Buffer) {
  const rates = Array.from({ length: GYRO_AXES }, (_unused, axis) =>
    Math.abs(data.readFloatLE(OFFSET.gyro + axis * FLOAT_BYTES))
  );
  return Math.max(...rates);
}

const { port, seconds } = readArgs();
const socket = dgram.createSocket("udp4");
const request = frameDsu({
  magic: "DSUC",
  sourceId: CLIENT_ID,
  event: DSU_EVENT.pad,
  body: Buffer.alloc(REQUEST_BODY_BYTES)
});
const pointerX: number[] = [];
const pointerY: number[] = [];
let gyro = 0;
let buttons = 0;
const endMs = Date.now() + seconds * MS_PER_SECOND;

while (Date.now() < endMs) {
  socket.send(request, port, LOOPBACK);
  const data = await nextDatagram({
    socket,
    timeoutMs: REPLY_TIMEOUT_MS
  });
  if (!data || data.length < PAD_PACKET_MIN_BYTES) {
    continue;
  }

  pointerX.push(data[OFFSET.pointerX] ?? 0);
  pointerY.push(data[OFFSET.pointerY] ?? 0);
  gyro = Math.max(gyro, peakGyro(data));
  buttons |= (data[OFFSET.buttons1] ?? 0) | ((data[OFFSET.buttons2] ?? 0) << BYTE_BITS);
}
socket.close();

if (pointerX.length === 0) {
  console.error(`no pad data from ${LOOPBACK}:${port} - is the bridge running and the controller tracked?`);
  process.exit(1);
}

const buttonsHex = buttons.toString(HEX_RADIX).padStart(HEX_DIGITS, "0");
console.log(
  `${pointerX.length} packets | pointer X ${Math.min(...pointerX)}-${Math.max(...pointerX)}  Y ${Math.min(...pointerY)}-${Math.max(...pointerY)} (128 = centre) | max gyro ${gyro.toFixed(0)} deg/s | buttons seen 0x${buttonsHex}`
);
