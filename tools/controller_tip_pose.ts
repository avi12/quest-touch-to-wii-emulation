// Ask SteamVR for the Touch controller "tip" (aim ray) transform relative to its tracked pose.
//
// On a Quest 2 this prints a forward direction of about (0, -0.61, -0.79) in the controller frame: SteamVR idea of
// where the controller "points". The bridge defaults to the controller -z axis instead, because that matches how a Wiimote is held.
import { OpenVr, PROPERTY } from "#/openvr.ts";
import type { ComponentTransform } from "#/openvr.ts";

const COMPONENTS = ["tip", "base", "handgrip"];
const NAME_COLUMN_WIDTH = 9;
const DECIMALS = 3;
const FORWARD_COLUMN = 2; // forward is -z: the negated third column of the rotation

function formatVector(values: readonly number[]) {
  return `[${values.map(value => Number(value.toFixed(DECIMALS)) + 0).join(" ")}]`;
}

function formatComponent({ name, transform }: { name: string; transform: ComponentTransform }) {
  const forward = transform.rot.map(row => -(row[FORWARD_COLUMN] ?? 0));
  return `${name.padEnd(NAME_COLUMN_WIDTH)} forward(-z) in controller frame: ${formatVector(forward)} origin: ${formatVector(transform.origin)}`;
}

const openVr = new OpenVr();
const hand = openVr.controllerIndex("right");
if (hand === openVr.invalid) {
  console.error("no right-hand controller: SteamVR does not see it (is it awake and tracked?)");
  process.exit(1);
}

const model = openVr.stringProperty({
  index: hand,
  property: PROPERTY.renderModelName
});
console.log("render model:", model);
for (const name of COMPONENTS) {
  const transform = openVr.componentTransform({
    index: hand,
    model,
    component: name
  });
  console.log(
    transform
      ? formatComponent({
          name,
          transform
        })
      : `${name.padEnd(NAME_COLUMN_WIDTH)} not available`
  );
}
