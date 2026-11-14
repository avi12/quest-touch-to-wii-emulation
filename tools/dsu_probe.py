"""Connect to a running bridge as a DSU client and print what Dolphin would see.

    python tools/dsu_probe.py [port] [seconds]

Prints the min/max of the pointer axes (128 = centre), the largest gyro rate and the pressed buttons.
Use it to check the bridge without Dolphin: move the controller while it runs.
"""
import socket
import struct
import sys
import time
import zlib

port = int(sys.argv[1]) if len(sys.argv) > 1 else 26760
seconds = float(sys.argv[2]) if len(sys.argv) > 2 else 5

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.settimeout(1)


def packet(event, body):
    payload = struct.pack("<I", event) + body
    hdr = b"DSUC" + struct.pack("<HHII", 1001, len(payload), 0, 1234)
    return hdr[:8] + struct.pack("<I", zlib.crc32(hdr + payload) & 0xFFFFFFFF) + hdr[12:] + payload


request = packet(0x100002, struct.pack("<BBB6s", 0, 0, 0, b"\0" * 6))
px, py, gyro, buttons, n = [], [], 0.0, 0, 0
end = time.time() + seconds
while time.time() < end:
    s.sendto(request, ("127.0.0.1", port))
    try:
        data, _ = s.recvfrom(2048)
    except socket.timeout:
        continue
    if len(data) < 100:
        continue
    n += 1
    px.append(data[42]); py.append(data[43])
    gyro = max(gyro, *(abs(v) for v in struct.unpack_from("<3f", data, 88)))
    buttons |= data[36] | (data[37] << 8)
if not n:
    sys.exit(f"no pad data from 127.0.0.1:{port} - is the bridge running and the controller tracked?")
print(f"{n} packets | pointer X {min(px)}-{max(px)}  Y {min(py)}-{max(py)} (128 = centre) | max gyro {gyro:.0f} deg/s | buttons seen 0x{buttons:04x}")
