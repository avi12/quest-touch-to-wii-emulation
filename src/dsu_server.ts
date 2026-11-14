// Minimal DSU (Cemuhook) pad server, speaking exactly what Dolphin's DSU client expects.
// Protocol: https://github.com/v1993/cemuhook-protocol
// Dolphin-specific behaviour (read from Dolphin's DualShockUDPClient.cpp) is noted where it matters.
import dgram from "node:dgram";
import zlib from "node:zlib";

import { z } from "zod";

// DSU digital button bits. Dolphin only reads Share/Options/L3/R3 from these; everything else comes from the analog fields.
export const BUTTONS_1 = {
  share: 0x01,
  l3: 0x02,
  r3: 0x04,
  options: 0x08,
  up: 0x10,
  right: 0x20,
  down: 0x40,
  left: 0x80
} as const;
export const BUTTONS_2 = {
  l2: 0x01,
  r2: 0x02,
  l1: 0x04,
  r1: 0x08,
  x: 0x10,
  a: 0x20,
  b: 0x40,
  y: 0x80
} as const;

export const DSU_EVENT = {
  version: 0x100000,
  ports: 0x100001,
  pad: 0x100002
} as const;
const DSU_HEADER_BYTES = 20;
const PROTOCOL_VERSION = 1001;
const SERVER_ID_BASE = 0x7011c0de;
const CLIENT_TIMEOUT_MS = 5000; // Dolphin re-requests the pad regularly; drop it after this long without a request
const RECENTER_HOLD_MS = 1000;
const EVENT_ID_BYTES = 4;
const PAD_HEAD_BYTES = 5; // [connected, packetNumber(4)]
const ANALOG_PRESSED = 255;
const STICK_MAX = 255;
const STICK_HALF_RANGE = 127.5;
const LOOPBACK = "127.0.0.1";
const MAGIC_REQUEST = "DSUC";
const MAGIC_RESPONSE = "DSUS";
const MAGIC_BYTES = 4;
const CRC_OFFSET = 8;
const SOURCE_ID_OFFSET = 12;
const EVENT_OFFSET = 16;
const LENGTH_OFFSET = 6;
const VERSION_OFFSET = 4;
// slot 0, state connected, model "full gyro", connection USB, fake MAC, battery full
const SLOT_INFO = Buffer.from([0, 2, 2, 1, 0, 0, 0, 0, 0, 1, 5]);
const TOUCHPAD_BYTES = 12;
const MOTION_TIMESTAMP_BYTES = 8;
const MOTION_FLOAT_BYTES = 4;
const MOTION_FLOAT_COUNT = 6;

const dsuEventSchema = z.enum(DSU_EVENT);
/** What a client request must look like before we act on it: the DSUC magic and a known event id. */
const requestSchema = z.object({
  magic: z.literal(MAGIC_REQUEST),
  event: dsuEventSchema
});

export type Vec3 = readonly [number, number, number];
export type Quad = readonly [number, number, number, number];
interface Client {
  address: string;
  port: number;
  lastSeenMs: number;
}

/** One pad snapshot: digital bits, stick / trigger bytes, microsecond timestamp, accel in g, gyro in deg/s. */
export interface PadState {
  buttons1: number;
  buttons2: number;
  sticks: Quad; // [LX, LY, RX, RY]
  triggers: Quad; // [R1, L1, R2, L2] analog
  tsUs: number;
  accel: Vec3;
  gyro: Vec3; // (pitch, yaw, roll)
}

/** Stick value -1..1 -> 0..255 (128 = centre). */
export function stickByte(value: number) {
  return Math.max(0, Math.min(STICK_MAX, Math.round((value + 1) * STICK_HALF_RANGE)));
}

/** One DSU datagram: 16 header bytes, CRC32 over the whole packet (CRC field zeroed while hashing), event id, body. */
export function frameDsu({
  magic,
  sourceId,
  event,
  body
}: {
  magic: string;
  sourceId: number;
  event: number;
  body: Buffer;
}) {
  const packet = Buffer.alloc(DSU_HEADER_BYTES + body.length);
  packet.write(magic, 0, MAGIC_BYTES, "latin1");
  packet.writeUInt16LE(PROTOCOL_VERSION, VERSION_OFFSET);
  packet.writeUInt16LE(EVENT_ID_BYTES + body.length, LENGTH_OFFSET);
  packet.writeUInt32LE(sourceId, SOURCE_ID_OFFSET);
  packet.writeUInt32LE(event, EVENT_OFFSET);
  body.copy(packet, DSU_HEADER_BYTES);
  packet.writeUInt32LE(zlib.crc32(packet), CRC_OFFSET);
  return packet;
}
function analogOf({ bits, mask }: { bits: number; mask: number }) {
  return bits & mask ? ANALOG_PRESSED : 0;
}

function motionBytes({ tsUs, accel, gyro }: Pick<PadState, "tsUs" | "accel" | "gyro">) {
  const motion = Buffer.alloc(MOTION_TIMESTAMP_BYTES + MOTION_FLOAT_COUNT * MOTION_FLOAT_BYTES);
  motion.writeBigUInt64LE(BigInt(tsUs), 0);
  for (const [i, value] of [...accel, ...gyro].entries()) {
    motion.writeFloatLE(value, MOTION_TIMESTAMP_BYTES + MOTION_FLOAT_BYTES * i);
  }
  return motion;
}

export class Server {
  readonly clients = new Map<string, Client>(); // "ip:port" -> client that asked for pad data
  recenterUntil = Date.now() + RECENTER_HOLD_MS; // while in the future, the PS button is held (= Dolphin's IMU "Recenter")
  readonly #serverId: number; // unique per instance
  readonly #socket = dgram.createSocket("udp4");
  #packetNumber = 0;

  constructor(port: number) {
    this.#serverId = (SERVER_ID_BASE + port) >>> 0;
    this.#socket.on("message", (data, remote) =>
      this.#onMessage({
        data,
        remote
      })
    );
    this.#socket.on("error", () => {}); // e.g. ICMP port-unreachable after a client quits
    this.#socket.bind(port, LOOPBACK);
  }

  #reply({ event, body, remote }: { event: number; body: Buffer; remote: dgram.RemoteInfo }) {
    this.#socket.send(
      frameDsu({
        magic: MAGIC_RESPONSE,
        sourceId: this.#serverId,
        event,
        body
      }),
      remote.port,
      remote.address
    );
  }

  #onMessage({ data, remote }: { data: Buffer; remote: dgram.RemoteInfo }) {
    if (data.length < DSU_HEADER_BYTES) {
      return;
    }

    const parsed = requestSchema.safeParse({
      magic: data.toString("latin1", 0, MAGIC_BYTES),
      event: data.readUInt32LE(EVENT_OFFSET)
    });
    if (!parsed.success) {
      return;
    }

    const { event } = parsed.data;
    if (event === DSU_EVENT.pad) {
      this.clients.set(`${remote.address}:${remote.port}`, {
        address: remote.address,
        port: remote.port,
        lastSeenMs: Date.now()
      });
      return;
    }

    if (event === DSU_EVENT.version) {
      const body = Buffer.alloc(2);
      body.writeUInt16LE(PROTOCOL_VERSION);
      this.#reply({
        event,
        body,
        remote
      });
      return;
    }

    this.#reply({
      event,
      body: Buffer.concat([SLOT_INFO, Buffer.from([0])]),
      remote
    });
  }

  /** sticks = [LX, LY, RX, RY] bytes; triggers = [R1, L1, R2, L2] analog bytes; accel in g (DSU axes), gyro in deg/s (pitch, yaw, roll). */
  sendPad({ buttons1, buttons2, sticks, triggers, tsUs, accel, gyro }: PadState) {
    // Dolphin reads the D-pad and the face buttons from the ANALOG fields, so mirror the digital bits there.
    const dpad = [BUTTONS_1.left, BUTTONS_1.down, BUTTONS_1.right, BUTTONS_1.up].map(mask =>
      analogOf({
        bits: buttons1,
        mask
      })
    );
    // Dolphin's field order is Square, Cross, Circle, Triangle (not the Y, B, A, X order of the spec).
    const face = [BUTTONS_2.x, BUTTONS_2.a, BUTTONS_2.b, BUTTONS_2.y].map(mask =>
      analogOf({
        bits: buttons2,
        mask
      })
    );
    this.#packetNumber = (this.#packetNumber + 1) >>> 0;
    const head = Buffer.alloc(PAD_HEAD_BYTES);
    head[0] = 1;
    head.writeUInt32LE(this.#packetNumber, 1);
    const isRecentering = Date.now() < this.recenterUntil;
    const flags = [buttons1, buttons2, isRecentering ? 1 : 0, 0];
    const body = Buffer.concat([
      SLOT_INFO,
      head,
      Buffer.from([...flags, ...sticks, ...dpad, ...face, ...triggers]),
      Buffer.alloc(TOUCHPAD_BYTES),
      motionBytes({
        tsUs,
        accel,
        gyro
      })
    ]);
    const packet = frameDsu({
      magic: MAGIC_RESPONSE,
      sourceId: this.#serverId,
      event: DSU_EVENT.pad,
      body
    });
    const now = Date.now();
    for (const [key, client] of this.clients) {
      if (now - client.lastSeenMs > CLIENT_TIMEOUT_MS) {
        this.clients.delete(key);
        continue;
      }

      this.#socket.send(packet, client.port, client.address);
    }
  }
}

// Short names kept for the bridge's imports.
