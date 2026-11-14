"""Ask SteamVR for the Touch controller's 'tip' (aim ray) transform relative to its tracked pose.

On a Quest 2 this prints a forward direction of about (0, -0.61, -0.79) in the controller frame: SteamVR's idea of
where the controller 'points'. The bridge defaults to the controller's -z axis instead, because that matches how a Wiimote is held.
"""
import numpy as np
import openvr

openvr.init(openvr.VRApplication_Background)
vr = openvr.VRSystem()
models = openvr.VRRenderModels()
hand = vr.getTrackedDeviceIndexForControllerRole(openvr.TrackedControllerRole_RightHand)
name = vr.getStringTrackedDeviceProperty(hand, openvr.Prop_RenderModelName_String)
_, state = vr.getControllerState(hand)
print("render model:", name)
for comp in ("tip", "base", "handgrip"):
    mode_state = openvr.RenderModel_ControllerMode_State_t()
    res = models.getComponentState(name, comp, state, mode_state)
    _, cs = res if isinstance(res, tuple) else (True, res)
    m = cs.mTrackingToComponentLocal.m
    M = np.array([[m[i][j] for j in range(4)] for i in range(3)])
    print(f"{comp:9s} forward(-z) in controller frame:", (M[:, :3] @ np.array([0, 0, -1])).round(3), "origin:", M[:, 3].round(3))
