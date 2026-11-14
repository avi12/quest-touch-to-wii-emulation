"""Find the axis a controller rolls around ('twist it like a screwdriver') - a clean measurement of how YOU hold it.

    python tools/calibrate_pointing_axis.py

Hold the controller aimed at the screen; after 10 s it records for 10 s while you twist it around its pointing direction
without changing where it points. Prints the axis in the controller frame; add it to CANDIDATES in touch_dsu.py to use it.
"""
import time

import numpy as np
import openvr

openvr.init(openvr.VRApplication_Background)
vr = openvr.VRSystem()
hand = vr.getTrackedDeviceIndexForControllerRole(openvr.TrackedControllerRole_RightHand)


def rotation():
    m = vr.getDeviceToAbsoluteTrackingPose(openvr.TrackingUniverseStanding, 0, openvr.k_unMaxTrackedDeviceCount)[hand].mDeviceToAbsoluteTracking
    return np.array([[m[i][j] for j in range(3)] for i in range(3)])


time.sleep(10)
print("recording - twist now", flush=True)
cov, n, prev, end = np.zeros((3, 3)), 0, rotation(), time.time() + 10
while time.time() < end:
    time.sleep(0.01)
    cur = rotation()
    if np.array_equal(cur, prev):
        continue
    d = prev.T @ cur                                  # rotation in the controller's own frame
    sk = np.array([d[2, 1] - d[1, 2], d[0, 2] - d[2, 0], d[1, 0] - d[0, 1]]) / 2
    if np.linalg.norm(sk) > 1e-4:
        cov += np.outer(sk, sk)
        n += 1
    prev = cur
w, v = np.linalg.eigh(cov)
print(f"{n} samples | axis {v[:, -1].round(3)} (sign is arbitrary) | purity {w[-1] / w.sum():.2f} (1.0 = a perfectly clean twist)")
