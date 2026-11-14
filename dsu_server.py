"""Minimal DSU (Cemuhook) pad server, speaking exactly what Dolphin's DSU client expects.

Protocol: https://github.com/v1993/cemuhook-protocol
Dolphin-specific behaviour (read from Dolphin's DualShockUDPClient.cpp) is noted where it matters.
"""
import socket
import struct
import time
import zlib

# DSU digital button bits. Dolphin only reads Share/Options/L3/R3 from these; everything else comes from the analog fields.
B1 = dict(share=0x01, l3=0x02, r3=0x04, options=0x08, up=0x10, right=0x20, down=0x40, left=0x80)
B2 = dict(l2=0x01, r2=0x02, l1=0x04, r1=0x08, x=0x10, a=0x20, b=0x40, y=0x80)

EV_VERSION, EV_PORTS, EV_PAD = 0x100000, 0x100001, 0x100002
CLIENT_TIMEOUT_S = 5          # Dolphin re-requests the pad regularly; drop it after this long without a request


def u8(v):
    """Stick value -1..1 -> 0..255 (128 = centre)."""
    return max(0, min(255, int(round((v + 1) * 127.5))))


class Server:
    def __init__(self, port):
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.bind(("127.0.0.1", port))
        self.sock.setblocking(False)
        self.sid = 0x7011C0DE + port       # server id, unique per instance
        self.clients = {}                  # addr -> time of last pad request
        self.pkt = 0
        self.recenter_until = time.time() + 1.0   # while in the future, the PS button is held (= Dolphin's IMU "Recenter")

    # --- framing -------------------------------------------------------------
    def wrap(self, event, body):
        payload = struct.pack("<I", event) + body
        hdr = b"DSUS" + struct.pack("<HHII", 1001, len(payload), 0, self.sid)
        crc = zlib.crc32(hdr + payload) & 0xFFFFFFFF
        return hdr[:8] + struct.pack("<I", crc) + hdr[12:] + payload

    def shared(self, connected=True):
        # slot 0, state connected, model "full gyro", connection USB, fake MAC, battery full
        return struct.pack("<BBBB6sB", 0, 2 if connected else 0, 2, 1, b"\x00\x00\x00\x00\x00\x01", 5)

    # --- requests ------------------------------------------------------------
    def poll(self):
        """Answer version / port-list requests and remember which clients want pad data."""
        while True:
            try:
                data, addr = self.sock.recvfrom(1024)
            except (BlockingIOError, ConnectionResetError):
                return
            if len(data) < 20 or data[:4] != b"DSUC":
                continue
            ev = struct.unpack_from("<I", data, 16)[0]
            if ev == EV_VERSION:
                self.sock.sendto(self.wrap(EV_VERSION, struct.pack("<H", 1001)), addr)
            elif ev == EV_PORTS:
                self.sock.sendto(self.wrap(EV_PORTS, self.shared() + b"\x00"), addr)
            elif ev == EV_PAD:
                self.clients[addr] = time.time()

    # --- pad data ------------------------------------------------------------
    def send_pad(self, buttons1, buttons2, sticks, triggers, ts_us, accel, gyro):
        """sticks = (LX, LY, RX, RY) bytes; triggers = (R1, L1, R2, L2) analog bytes;
        accel in g (DSU axes), gyro in deg/s as (pitch, yaw, roll)."""
        # Dolphin reads the D-pad and the face buttons from the ANALOG fields, so mirror the digital bits there.
        dpad = bytes(255 if buttons1 & m else 0 for m in (B1['left'], B1['down'], B1['right'], B1['up']))
        # Dolphin's field order is Square, Cross, Circle, Triangle (not the Y, B, A, X order of the spec).
        face = bytes(255 if buttons2 & m else 0 for m in (B2['x'], B2['a'], B2['b'], B2['y']))
        self.pkt += 1
        body = self.shared() + struct.pack("<BI", 1, self.pkt)
        body += struct.pack("<BBBB", buttons1, buttons2, 1 if time.time() < self.recenter_until else 0, 0)
        body += bytes(sticks)                       # LX LY RX RY (Dolphin's "Y" fields are 255 = up)
        body += dpad + face
        body += bytes(triggers)                     # R1 L1 R2 L2
        body += b"\x00" * 12                        # touchpad (unused)
        body += struct.pack("<Q", ts_us) + struct.pack("<3f", *accel) + struct.pack("<3f", *gyro)
        pkt = self.wrap(EV_PAD, body)
        now = time.time()
        for addr, seen in list(self.clients.items()):
            if now - seen > CLIENT_TIMEOUT_S:
                del self.clients[addr]
            else:
                self.sock.sendto(pkt, addr)
