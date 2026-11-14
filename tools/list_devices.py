"""List what SteamVR currently tracks: device class, controller role, pose validity, tracking result, position.

A controller that SteamVR cannot see shows up as 'valid 0' or is missing; 'pos' frozen while it moves means the headset
cameras have lost it (rotation still updates from the controller's IMU)."""
import openvr

openvr.init(openvr.VRApplication_Background)
vr = openvr.VRSystem()
poses = vr.getDeviceToAbsoluteTrackingPose(openvr.TrackingUniverseStanding, 0, openvr.k_unMaxTrackedDeviceCount)
for i in range(openvr.k_unMaxTrackedDeviceCount):
    if vr.isTrackedDeviceConnected(i):
        p = poses[i]
        m = p.mDeviceToAbsoluteTracking
        print(i, "class", vr.getTrackedDeviceClass(i), "role", vr.getControllerRoleForTrackedDeviceIndex(i),
              "valid", int(p.bPoseIsValid), "result", p.eTrackingResult, "pos", [round(m[r][3], 2) for r in range(3)])
