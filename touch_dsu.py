"""Quest Touch controllers -> Dolphin Wii Remotes, as DSU (Cemuhook) pads.

Reads the Touch controllers through SteamVR (OpenVR) and serves one DSU pad on UDP 26760.
The right controller is Wii Remote 1: aimed pointer, roll, swings, buttons; the left stick drives a Nunchuk.
A second copy (BRIDGE_PORT=26761 BRIDGE_HAND=left) turns the left controller into Wii Remote 2.

Needs SteamVR running with the headset connected (Air Link / Link / Virtual Desktop). No VR display is used.
Hotkeys (global, Ctrl+Alt so they never clash with Dolphin's): L toggles the left controller, ] and [ cycle the pointing axis.
Thumbstick click = recenter.  See README.md and docs/how-it-works.md.
"""
import ctypes, math, os, threading, time, winsound
import numpy as np
import openvr
from dsu_server import B1, B2, Server, u8

PORT = int(os.environ.get('BRIDGE_PORT', 26760))
PLAYER2 = os.environ.get('BRIDGE_HAND') == 'left'   # second instance: the LEFT Touch controller is its own Wiimote (Wii Remote 2)
HZ = 250
G = 9.80665
FLIP_ACCEL = (1, 1, 1)   # tweak if Dolphin shows an axis inverted
FLIP_GYRO = (1, 1, 1)
DYN_GAIN = 1.0    # swing acceleration passed to Dolphin (gated below so gentle aiming cannot bend the motion pointer's pitch)
SWING_LO, SWING_HI = 0.6, 1.2   # g: below LO the motion is ignored (aiming), above HI it is passed in full
ROLL_GAIN = 1.0   # 1 = hand follows the controller's roll 1:1; negative flips the direction
YAW_RANGE = math.radians(28)    # controller swing (each way) that spans the whole screen width
PITCH_RANGE = math.radians(18)
PITCH_BIAS = 0.0      # keep 0: a recenter must put the hand exactly in the middle of the screen
F0 = np.array([0.0, 0.0, -1.0])  # pointing axis = the controller's -z axis (chosen live with the Ctrl+Alt+[ ] selector; grip-dependent)
_u = np.array([0.0, 1.0, 0.0]) if abs(F0[1]) < 0.9 else np.array([0.0, 0.0, 1.0]); _u = _u - (_u @ F0) * F0; _u /= np.linalg.norm(_u)   # any axis not parallel to the aim
ROLL_QUARTER_TURNS = 0     # 0..3: rotates the Wiimote 'up' axis about the pointing axis (tunes the cursor hand's rotation)
for _ in range(ROLL_QUARTER_TURNS): _u = np.cross(F0, _u)
BASIS = (np.cross(F0, _u), F0, _u)   # right, forward, up (in the controller frame)

class OneEuro:
    # smooths jitter when slow, stays responsive when fast
    def __init__(self, mincut=0.8, beta=0.6, dcut=1.0): self.mc, self.b, self.dc = mincut, beta, dcut; self.x = None; self.dx = 0.0
    @staticmethod
    def _a(cut, dt): return 1.0 / (1.0 + 1.0 / (2 * math.pi * cut * dt))
    def __call__(self, v, dt):
        if self.x is None: self.x = v; return v
        d = (v - self.x) / dt
        self.dx += self._a(self.dc, dt) * (d - self.dx)
        self.x += self._a(self.mc + self.b * abs(self.dx), dt) * (v - self.x)
        return self.x
fx, fy = OneEuro(), OneEuro()
def beep(freq, ms=120): threading.Thread(target=winsound.Beep, args=(freq, ms), daemon=True).start()   # audible feedback (second controller connect / disconnect)
CANDIDATES = [   # (name, pointing axis in the controller frame)
    ("measured twist axis", (-0.134, 0.991, 0.025)), ("+x", (1, 0, 0)), ("-x", (-1, 0, 0)), ("+y", (0, 1, 0)), ("-y", (0, -1, 0)),
    ("+z", (0, 0, 1)), ("-z", (0, 0, -1)), ("Meta aim ray (0,-.61,-.79)", (0, -0.607, -0.794)), ("Meta aim ray reversed", (0, 0.607, 0.794)),
]
cand_i = 6; prev_f9 = False; prev_f8 = False
left_state = False; left_change_t = time.time()   # second controller connect/disconnect beep state
left_on = True; prev_f10 = False      # Ctrl+Alt+L toggles the left controller (Nunchuk / Wii Remote 2) on/off live
ref = None; prev_l3 = False; edge_t = 0
W_REF = np.array([0.0, 0.0, -1.0]); RIGHT_REF = np.array([1.0, 0.0, 0.0]); UP_REF = np.array([0.0, 1.0, 0.0])
def to_np(m):
    return np.array([[m[r][c] for c in range(4)] for r in range(3)], dtype=float)

def ctrl_state(vr, idx):
    ok, st = vr.getControllerState(idx)
    if not ok:
        return None
    pressed = st.ulButtonPressed
    bit = lambda b: bool(pressed & (1 << b))
    return dict(
        stick=(st.rAxis[0].x, st.rAxis[0].y), trig=st.rAxis[1].x, grip=st.rAxis[2].x,
        a=bit(openvr.k_EButton_A), b=bit(openvr.k_EButton_ApplicationMenu),
        click=bit(openvr.k_EButton_Axis0), sys=bit(openvr.k_EButton_System))

def main():
    global ref, prev_l3, edge_t, BASIS, W_REF, RIGHT_REF, UP_REF, F0, cand_i, prev_f9, prev_f8, left_on, prev_f10, left_state, left_change_t
    openvr.init(openvr.VRApplication_Background)
    vr = openvr.VRSystem()
    srv = Server(PORT)
    print("Bridge up on UDP", PORT, "(left controller = Wii Remote 2)" if PLAYER2 else "(right controller = Wii Remote 1)", "- waiting for controllers")
    prev_v, prev_t = None, None
    last_lin = np.zeros(3); last_w = np.zeros(3); last_pkt = None
    acc_s = np.zeros(3)
    dt = 1.0 / HZ
    nxt = time.perf_counter()
    last_report = 0
    while True:
        nxt += dt
        srv.poll()
        poses = vr.getDeviceToAbsoluteTrackingPose(openvr.TrackingUniverseStanding, 0, openvr.k_unMaxTrackedDeviceCount)
        r = vr.getTrackedDeviceIndexForControllerRole(openvr.TrackedControllerRole_LeftHand if PLAYER2 else openvr.TrackedControllerRole_RightHand)
        l = openvr.k_unTrackedDeviceIndexInvalid if PLAYER2 else vr.getTrackedDeviceIndexForControllerRole(openvr.TrackedControllerRole_LeftHand)
        if r == openvr.k_unTrackedDeviceIndexInvalid or not poses[r].bPoseIsValid:
            if time.time() - last_report > 3:
                print(("left" if PLAYER2 else "right"), "controller not tracked - holding last pose (keep it in front of the headset cameras)", flush=True); last_report = time.time()
            if last_pkt is not None:      # keep the pad alive with the last good data, motion zeroed
                srv.send_pad(*last_pkt)
            time.sleep(dt); continue
        seen2 = (not PLAYER2) and l != openvr.k_unTrackedDeviceIndexInvalid and bool(poses[l].bPoseIsValid)
        if not PLAYER2:
            if seen2 != left_state:
                if time.time() - left_change_t > 1.0:                     # must hold for a second (ignores tracking blips)
                    left_state = seen2; beep(1200 if seen2 else 500, 200); print("SECOND CONTROLLER", "CONNECTED" if seen2 else "DISCONNECTED", flush=True)
            else:
                left_change_t = time.time()
        p = poses[r]
        R = to_np(p.mDeviceToAbsoluteTracking)[:, :3]
        pos = to_np(p.mDeviceToAbsoluteTracking)[:, 3]
        now = time.perf_counter()
        lin = last_lin; w_world = last_w
        if prev_v is None:
            prev_v = (pos, None, R); prev_t = now
        elif not (np.array_equal(pos, prev_v[0]) and np.array_equal(R, prev_v[2])):   # a new pose arrived
            h = now - prev_t
            if h > 1e-3:
                v = (pos - prev_v[0]) / h
                lin = (v - prev_v[1]) / h if prev_v[1] is not None else np.zeros(3)
                dR = R @ prev_v[2].T                       # world-frame rotation since the last pose
                sk = np.array([dR[2,1]-dR[1,2], dR[0,2]-dR[2,0], dR[1,0]-dR[0,1]]) / 2   # = sin(angle) * axis
                n = np.linalg.norm(sk)
                w_world = (sk / n * math.asin(min(1.0, n)) / h) if n > 1e-12 else np.zeros(3)
                if np.linalg.norm(w_world) > math.radians(1200):     # tracking jump, not real motion
                    w_world = last_w; lin = last_lin
                prev_v = (pos, v, R); prev_t = now
                last_lin, last_w = lin, w_world
        acc_s = acc_s * 0.5 + lin * 0.5
        # accelerometer reads specific force (what a Wiimote feels): a - g, g = (0,-G,0)
        spec_world = acc_s + np.array([0, G, 0])
        spec_dev = R.T @ spec_world / G
        w_dev = R.T @ w_world
        # Controller frame -> Wiimote frame (x right, y forward, z up) using the measured pointing axis,
        # then -> Dolphin's DSU convention (read from Dolphin's DualShockUDPClient.cpp):
        #   accel (g): x = -right, y = -up, z = forward;  gyro (deg/s): pitch = +about right, yaw = -about up, roll = +about forward
        wm = lambda v: np.array([BASIS[0] @ v, BASIS[1] @ v, BASIS[2] @ v])   # (right, forward, up)
        ar, af, au = wm(spec_dev)
        wr, wf, wu = wm(w_dev) * math.degrees(1)
        # Dolphin's motion-sensor pointer turns the hand by the Wiimote's roll. Feed it ROLL ONLY (aim comes from the pointer above):
        # gravity is replaced by a version rotated by the roll about the aim axis, yaw/pitch rates are dropped, swings stay.
        gwm = wm(R.T @ np.array([0.0, 1.0, 0.0]))                 # world-up in the Wiimote frame (right, forward, up)
        phi = math.atan2(gwm[0], gwm[2]) * ROLL_GAIN               # roll about the aim axis, 0 = upright at the last recenter
        gv = np.array([math.sin(phi), 0.0, math.cos(phi)])
        dyn = np.array([ar, af, au]) - gwm                           # real motion acceleration (gravity removed), in g
        mag = float(np.linalg.norm(dyn))
        k = min(1.0, max(0.0, (mag - SWING_LO) / (SWING_HI - SWING_LO)))   # 0 while aiming gently, 1 for a real swing
        arv, afv, auv = gv + DYN_GAIN * k * dyn                      # roll-only gravity + the swing, only when it is a swing
        accel = np.array([-arv, -auv, afv]) * np.array(FLIP_ACCEL)
        gyro = np.array([0.0, 0.0, wf]) * np.array(FLIP_GYRO)      # roll rate only
        sr = ctrl_state(vr, r); sl = ctrl_state(vr, l) if l != openvr.k_unTrackedDeviceIndexInvalid else None
        b1 = b2 = 0
        sr = sr or dict(stick=(0, 0), trig=0, grip=0, a=0, b=0, click=0, sys=0)
        sl = sl or dict(stick=(0, 0), trig=0, grip=0, a=0, b=0, click=0, sys=0)
        ca = bool(ctypes.windll.user32.GetAsyncKeyState(0x11) & 0x8000) and bool(ctypes.windll.user32.GetAsyncKeyState(0x12) & 0x8000)   # Ctrl+Alt held
        f10 = ca and bool(ctypes.windll.user32.GetAsyncKeyState(0x4C) & 0x8000)   # L
        if f10 and not prev_f10:
            left_on = not left_on; print("LEFT CONTROLLER", "ON" if left_on else "OFF", flush=True)
        prev_f10 = f10
        if not left_on:
            sl = dict(stick=(0, 0), trig=0, grip=0, a=0, b=0, click=0, sys=0)    # ignore the left controller entirely
        if PLAYER2 and not left_on:
            sr = dict(stick=(0, 0), trig=0, grip=0, a=0, b=0, click=0, sys=0)    # Wii Remote 2 switched off: no buttons
        if sr["a"]: b2 |= B2["a"]
        if sr["b"]: b2 |= B2["b"]
        if sl["a"]: b2 |= B2["x"]
        if sl["b"]: b2 |= B2["y"]
        if sr["trig"] > 0.5: b2 |= B2["r2"]
        if sl["trig"] > 0.5: b2 |= B2["l2"]
        if sr["grip"] > 0.5: b2 |= B2["r1"]
        if sl["grip"] > 0.5: b2 |= B2["l1"]
        if sr["click"]: b1 |= B1["r3"]
        if sl["click"]: b1 |= B1["l3"]
        if sl["sys"]: b1 |= B1["options"]
        if sr["sys"]: b1 |= B1["share"]
        # absolute pointer from where the controller aims; recenter on first sample, on a thumbstick click, or when pinned at an edge
        d = R @ F0
        ang = (math.atan2(d[0], -d[2]), math.asin(max(-1, min(1, d[1] / np.linalg.norm(d)))))
        f9 = ca and not PLAYER2 and bool(ctypes.windll.user32.GetAsyncKeyState(0xDD) & 0x8000); f8 = ca and not PLAYER2 and bool(ctypes.windll.user32.GetAsyncKeyState(0xDB) & 0x8000)   # ] and [
        if (f9 and not prev_f9) or (f8 and not prev_f8):          # Ctrl+Alt+] next / Ctrl+Alt+[ previous pointing-axis candidate
            cand_i = (cand_i + (1 if f9 and not prev_f9 else -1)) % len(CANDIDATES)
            F0 = np.array(CANDIDATES[cand_i][1], float); F0 /= np.linalg.norm(F0); ref = None
            print("CANDIDATE", cand_i, CANDIDATES[cand_i][0], flush=True)
        prev_f9, prev_f8 = f9, f8
        l3 = bool(sl["click"] or sr["click"])   # either thumbstick click recenters
        if abs(fx.x or 0) >= 0.999 or abs(fy.x or 0) >= 0.999:      # pinned at the screen edge: you're not aiming there
            edge_t = edge_t or time.time()
            if time.time() - edge_t > 1.5 and abs((R @ F0)[1]) < 0.95: ref = None; edge_t = 0   # only if the aim is usable (not pointing straight up/down)
        else:
            edge_t = 0
        if ref is None or (l3 and not prev_l3):
            ref = ang; fx.x = fy.x = None
            dn_ = d / np.linalg.norm(d)                       # where the screen is: your aim at recenter, pitch included
            if abs(dn_[1]) < 0.95:
                W_REF = dn_; RIGHT_REF = np.cross(W_REF, np.array([0.0, 1.0, 0.0])); RIGHT_REF /= np.linalg.norm(RIGHT_REF)
                UP_REF = np.cross(RIGHT_REF, W_REF)               # screen plane axes (right, up) and its normal (W_REF)
            uc = R.T @ np.array([0.0, 1.0, 0.0])           # world up, in the controller frame
            uc = uc - (uc @ F0) * F0                        # drop the part along the pointing axis
            if np.linalg.norm(uc) > 0.2:                    # skip if pointing straight up/down
                uc /= np.linalg.norm(uc); BASIS = (np.cross(F0, uc), F0, uc)   # hand is upright at recenter, whatever the grip
            print('RECENTER', flush=True); srv.recenter_until = time.time() + 0.3
        prev_l3 = l3
        dn = max(0.1, float(d @ W_REF) / np.linalg.norm(d))
        sx, sy = (d @ RIGHT_REF) / np.linalg.norm(d) / dn, (d @ UP_REF) / np.linalg.norm(d) / dn   # where the aim ray hits a flat screen
        px = max(-1.0, min(1.0, fx(sx / math.tan(YAW_RANGE), dt)))
        py = max(-1.0, min(1.0, fy(sy / math.tan(PITCH_RANGE), dt) + PITCH_BIAS))
        # right stick -> D-pad bits (the stick axes now carry the pointer)
        rx, ry = sr["stick"]
        if ry > 0.6: b1 |= B1["up"]
        if ry < -0.6: b1 |= B1["down"]
        if rx > 0.6: b1 |= B1["right"]
        if rx < -0.6: b1 |= B1["left"]
        if os.environ.get('PTR_TEST'):       # diagnostic: sweep known pointer values, ignoring the controller
            px, py = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)][int(time.time() / 4) % 5]
            accel = np.array([0.0, -1.0, 0.0]); gyro = np.zeros(3)   # level, still Wiimote
        if PLAYER2 and not left_on: px, py = -1.0, 0.0                          # park the hand off-screen
        sticks = (u8(sl["stick"][0]), u8(sl["stick"][1]), u8(px), u8(py))
        trig = (int(sr["grip"] * 255), int(sl["grip"] * 255), int(sr["trig"] * 255), int(sl["trig"] * 255))
        last_pkt = (b1, b2, sticks, trig, int(time.time() * 1e6), accel.tolist(), [0.0, 0.0, 0.0])
        srv.send_pad(b1, b2, sticks, trig, int(time.time() * 1e6), accel.tolist(), gyro.tolist())
        if time.time() - last_report > 3:
            print("left controller:", "ON (used)" if left_on else "OFF (ignored)", "| seen by SteamVR:", bool(l != openvr.k_unTrackedDeviceIndexInvalid and poses[l].bPoseIsValid), "| dolphin connected:", len(srv.clients) > 0, "| ptr", round(px, 2), round(py, 2), flush=True); last_report = time.time()
        d = nxt - time.perf_counter()
        if d > 0: time.sleep(d)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
