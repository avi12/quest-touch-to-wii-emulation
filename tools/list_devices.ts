// List what SteamVR currently tracks: device class, controller role, pose validity, tracking result, position.
//
// A controller that SteamVR cannot see shows up as 'valid 0' or is missing; 'pos' frozen while it moves means the headset
// cameras have lost it (rotation still updates from the controller IMU).
import { MAX_DEVICES, OpenVr } from "#/openvr.ts";

const POSITION_DECIMALS = 2;

function formatNumber(value: number) {
  const rounded = Number(value.toFixed(POSITION_DECIMALS));
  return Number.isInteger(rounded) ? rounded.toFixed(1) : String(rounded);
}

const openVr = new OpenVr();
const poseOf = openVr.poses();
for (let index = 0; index < MAX_DEVICES; index++) {
  if (!openVr.isConnected(index)) {
    continue;
  }

  const pose = poseOf(index);
  if (!pose) {
    continue;
  }

  const position = `[${pose.pos.map(formatNumber).join(", ")}]`;
  console.log(
    `${index} class ${openVr.deviceClass(index)} role ${openVr.controllerRole(index)} valid ${Number(pose.valid)} result ${pose.trackingResult} pos ${position}`
  );
}
