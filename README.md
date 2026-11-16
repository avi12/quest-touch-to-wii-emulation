# Touch → Dolphin

Play Wii games in [Dolphin](https://dolphin-emu.org) on a normal monitor with **Meta Quest Touch controllers** as real Wii Remotes: aimed pointer, roll, swings, buttons, and a second player on the left controller. No VR display is used.

```mermaid
flowchart LR
    Q["Quest 2 + Touch controllers"] -- "Air Link / Link / Virtual Desktop" --> S["SteamVR<br/>(tracking only)"]
    S -- OpenVR --> B["touch_dsu.ts<br/>pointer, roll, swing, buttons"]
    B -- "DSU / Cemuhook, UDP 26760" --> D1["Dolphin: Wii Remote 1<br/>right controller"]
    B2["touch_dsu.ts (left)"] -- "UDP 26761" --> D2["Dolphin: Wii Remote 2<br/>left controller"]
    S -- OpenVR --> B2
```

The headset only _tracks_ the controllers. SteamVR hands their poses to a small Node.js (TypeScript) bridge, which pretends to be a DualShock-style pad with motion sensors (the DSU protocol Dolphin already understands). Nothing in Dolphin is patched.

## Requirements

- Windows, Node.js 26+, pnpm, Dolphin (tested on 2609b)
- A Quest (tested: Quest 2) connected to the PC through **Air Link, Link or Virtual Desktop**, with SteamVR running
- `pnpm install` (OpenVR is loaded through `koffi`; no Python needed)

## Quick start

1. Connect the headset to the PC and start SteamVR. Wake both controllers (press a button) and hold them in front of the headset cameras.
2. **Close Dolphin**, then copy the files in `config/` into `%APPDATA%\Dolphin Emulator\Config\` (merge `WiimoteNew.ini`; see [Dolphin setup](#dolphin-setup)).
3. Start the bridges: `scripts\start.ps1` (stop them with `scripts\stop.ps1`).
4. Open Dolphin and a Wii game. Aim at the middle of the screen and **click the right thumbstick** to recenter.

## Controls

| Touch               | Wii Remote                  |
| ------------------- | --------------------------- |
| Aim the controller  | Pointer                     |
| Roll the controller | Hand tilts                  |
| Fast swing          | Motion (tennis, bowling, …) |
| A                   | A                           |
| Right grip          | B                           |
| B                   | 1                           |
| Y                   | 2                           |
| X                   | −                           |
| Right trigger       | Home                        |
| Left menu           | +                           |
| Right thumbstick    | D-pad                       |
| Thumbstick click    | Recenter                    |

With a Nunchuk attached to Wii Remote 1 in Dolphin (Controllers → Wii Remote 1 → Extension), the left controller acts as the Nunchuk: stick = stick, grip = C, trigger = Z, and its motion drives the Nunchuk's accelerometer, so Boxing punches work with both hands. **Ctrl+Alt+L** switches the left controller on and off while you play (off: the Nunchuk and Wii Remote 2 sit idle). For two players, use Wii Remote 2 on the left controller instead.

**Hotkeys** (global, Ctrl+Alt so they never clash with Dolphin's own F-keys):

| Keys           | Action                                                                         |
| -------------- | ------------------------------------------------------------------------------ |
| Ctrl+Alt+L     | Switch the left controller (Nunchuk / Wii Remote 2) on or off                  |
| Ctrl+Alt+] / [ | Cycle the pointing-axis candidate (only if aiming feels rotated for your grip) |

One beep: high = second controller connected, low = disconnected.

## Dolphin setup

Dolphin rewrites its config on exit, so **edit config files only while Dolphin is closed.**

| File (`%APPDATA%\Dolphin Emulator\Config\`) | What to do                                                                                         |
| ------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `DSUClient.ini`                             | Copy `config/DSUClient.ini` (two servers: right and left bridge)                                   |
| `WiimoteNew.ini`                            | Merge `[Wiimote1]` and `[Wiimote2]` from `config/WiimoteNew.ini`                                   |
| `Dolphin.ini`                               | Under `[Input]` set `BackgroundInput = True` so the game keeps input when another window has focus |

In Dolphin, Controllers → Wii Remote 1 and 2 must be **Emulated Wii Remote**. Leave **Extension: None** unless a game needs a Nunchuk (Wii Sports asks you to unplug it).

## Troubleshooting

| Symptom                         | Cause / fix                                                                                                                                                   |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hand doesn't appear             | Pointer pinned off-screen. Aim at the screen and click the right thumbstick.                                                                                  |
| Controllers vanish from SteamVR | They sleep when idle and need the headset cameras. Press a button; run `tools\keep_headset_awake.ps1` so the headset itself doesn't sleep.                    |
| Pointer drifts / jitters        | The headset can't see the controller (position is frozen, only the gyro is working). Put the headset on a stand facing you, 1–2 m away, controllers in front. |
| Emulator froze after a hotkey   | F10 is Dolphin's **Pause** (F9 = screenshot). Press F10 again. The bridge hotkeys use Ctrl+Alt for this reason.                                               |
| Face buttons wrong              | Dolphin reads face buttons from the analog fields in Square, Cross, Circle, Triangle order; see `dsu_server.py`.                                              |
| Nothing at all                  | `node tools\dsu_probe.ts` shows whether the bridge produces data; `node tools\list_devices.ts` shows what SteamVR sees.                                       |

## Layout

```
touch_dsu.ts        the bridge: poses -> pointer, roll, swings, buttons -> DSU pad
dsu_server.ts       the DSU (Cemuhook) server, with Dolphin's packet quirks
scripts/            start.ps1 / stop.ps1 (PID-based, never kills by name)
config/             Dolphin config for Wii Remote 1 and 2
tools/              diagnostics: dsu_probe, list_devices, controller_tip_pose, calibrate_pointing_axis, keep_headset_awake
docs/how-it-works.md   the findings behind every design choice
```

## Limits

- Needs the headset and SteamVR; the Touch controllers cannot be paired to a PC directly (they only bond to their headset).
- Tracking quality is the headset's: out of its camera view the controller falls back to gyro-only rotation.
- MotionPlus games (e.g. Wii Sports Resort) are untested: the bridge sends roll-only gyro data.
- Tested on Quest 2, Windows 11, Dolphin 2609b, Wii Sports.

## Development

- `pnpm start` runs one bridge; `pnpm check` type-checks (tsgo); `pnpm lint` / `pnpm lint:fix` run the avi12 lint (oxlint, oxfmt, naming rules in the youtube-time-manager style); `pnpm lint:all` adds types and fallow.
- See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Support

[GitHub Sponsors](https://github.com/sponsors/avi12) or [PayPal](https://paypal.me/avi12).

## License

[Apache-2.0](LICENSE)
