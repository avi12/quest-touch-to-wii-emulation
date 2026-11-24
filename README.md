# Touch → Dolphin

Play Wii games in [Dolphin](https://dolphin-emu.org) on a normal monitor with **Meta Quest Touch controllers** as real Wii Remotes: aimed pointer, roll, swings, buttons, and a second player on the left controller. No VR display is used.

```mermaid
flowchart LR
    Q["Quest 2 + Touch controllers"] -- "Air Link / Link / Virtual Desktop" --> S["SteamVR<br/>(tracking only)"]
    S -- OpenVR --> B["touch_dsu<br/>pointer, roll, swing, buttons"]
    B -- "DSU / Cemuhook, UDP 26760" --> D1["Dolphin: Wii Remote 1<br/>right controller"]
    B2["touch_dsu (left)"] -- "UDP 26761" --> D2["Dolphin: Wii Remote 2<br/>left controller"]
    S -- OpenVR --> B2
```

The headset only _tracks_ the controllers. SteamVR hands their poses to a small Rust bridge, which pretends to be a DualShock-style pad with motion sensors (the DSU protocol Dolphin already understands). Nothing in Dolphin is patched.

## Requirements

- Windows, Rust (stable, via rustup), Dolphin (tested on 2609b)
- A Quest (tested: Quest 2) connected to the PC through **Air Link, Link or Virtual Desktop**, with SteamVR running
- Nothing else: `openvr_api.dll` is loaded from the SteamVR install at run time, and the bridge has no third-party crates

## Quick start

Once:

1. **Close Dolphin**, then copy the files in `config/` into `%APPDATA%\Dolphin Emulator\Config\` (merge `WiimoteNew.ini`; see [Dolphin setup](#dolphin-setup)).
2. Put your Dolphin and game paths in `.play.json` in the repo root (gitignored):
   `{ "dolphin": "C:\…\Dolphin.exe", "disc": "C:\…\Wii Sports.rvz" }`

Every time:

1. Connect the headset (Air Link). Wake both controllers (press a button) and hold them in front of the headset cameras.
2. Run `scripts\play.ps1`. It starts SteamVR and both bridges if needed, then Dolphin's Wii Menu with the game already in the drive (Disc Channel). `-Disc <file>` picks another game; `-Telemetry` records the motion for diagnosis.
3. Aim at the middle of the screen and **click the right thumbstick** to recenter.
4. When done: close Dolphin, then `scripts\stop.ps1`.

Without the launcher: start SteamVR, run `scripts\start.ps1` for the bridges, then open Dolphin and a game yourself. The Wii Menu needs the system software once: Dolphin → **Tools → Perform Online System Update**.

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

Both thumbsticks have a 15% dead zone, so a stick at rest reads exactly centred.

The left controller acts as a Nunchuk on Wii Remote 1: stick = stick, grip = C, trigger = Z, and its motion drives the Nunchuk's accelerometer, so Boxing punches work with both hands. **Ctrl+Alt+L** switches the left controller on and off while you play, and the game sees the Nunchuk being plugged in or pulled out (the bridge sends L3 while the left controller is on and tracked, and `config/WiimoteNew.ini` binds Wii Remote 1's `Extension` to it). For two players, use Wii Remote 2 on the left controller instead.

**Hotkeys** (global, Ctrl+Alt so they never clash with Dolphin's own F-keys; either Ctrl and either Alt work, so Right Ctrl + Right Alt does too):

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

In Dolphin, Controllers → Wii Remote 1 and 2 must be **Emulated Wii Remote**. Leave **Extension: None** unless a game needs a Nunchuk (Wii Sports asks you to unplug it), and untick **Attach MotionPlus**: the bridge sends roll-only gyro data.

Keep **Motion Simulation → Point** at Dolphin's defaults (Vertical Offset 10 cm, Total Yaw 25°, Total Pitch 20°), and make **Config → Wii → Sensor Bar Position** match what you told the Wii Menu (Top = above the TV). The Wii shifts the cursor by its sensor-bar setting and Dolphin's offset cancels that only when the two agree.

## Troubleshooting

| Symptom                                              | Cause / fix                                                                                                                                                                                                                           |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Hand doesn't appear                                  | Pointer pinned off-screen. Aim at the screen and click the right thumbstick.                                                                                                                                                          |
| Controllers vanish from SteamVR                      | They sleep when idle and need the headset cameras. Press a button; run `tools\keep_headset_awake.ps1` so the headset itself doesn't sleep.                                                                                            |
| Pointer drifts / jitters                             | The headset can't see the controller (position is frozen, only the gyro is working). Put the headset on a stand facing you, 1–2 m away, controllers in front.                                                                         |
| Emulator froze after a hotkey                        | F10 is Dolphin's **Pause** (F9 = screenshot). Press F10 again. The bridge hotkeys use Ctrl+Alt for this reason.                                                                                                                       |
| Face buttons wrong                                   | Dolphin reads face buttons from the analog fields in Square, Cross, Circle, Triangle order; see `crates/dsu`.                                                                                                                         |
| Hand moves but no button works                       | SteamVR is withholding button input: a headset menu (Meta's or SteamVR's dashboard) has focus. Close it; if that fails, restart SteamVR and the bridges. `cargo run -p tools --bin controller_buttons` shows "no state" in this case. |
| Recenter lands off-centre, top or bottom unreachable | Dolphin's Sensor Bar Position disagrees with the Wii Menu's; see [Dolphin setup](#dolphin-setup).                                                                                                                                     |
| Wii Menu shows no game                               | Start it with `scripts\play.ps1`, which inserts the disc; or Dolphin → Emulation → Change Disc.                                                                                                                                       |
| Headset restarted by itself                          | Battery ran out (Air Link drains it faster than USB charges). Charge it, then reconnect; wireless adb needs `adb tcpip 5555` again over USB.                                                                                          |
| Nothing at all                                       | `cargo run -p tools --bin dsu_probe` shows whether the bridge produces data; `cargo run -p tools --bin list_devices` shows what SteamVR sees.                                                                                         |

## Layout

```
crates/bridge       the bridge (binary touch_dsu): poses -> pointer, roll, swings, buttons -> DSU pad
crates/dsu          the DSU (Cemuhook) server, with Dolphin's packet quirks
crates/openvr-sys   SteamVR's openvr_api.dll, loaded at run time (unsafe FFI lives only here and in win32-sys)
crates/win32-sys    key state, beep, timer resolution
crates/geometry     Vec3 / Mat3, shared by every crate
crates/numeric      the one checked float-to-integer conversion, JavaScript-style number formatting
crates/tools        diagnostics: dsu_probe, list_devices, controller_buttons, controller_tip_pose, calibrate_pointing_axis, telemetry_tail
scripts/            play.ps1 (everything in one step), start.ps1 / stop.ps1 (bridges; PID-based, never kills by name)
config/             Dolphin config for Wii Remote 1 and 2
tools/              keep_headset_awake.ps1
docs/how-it-works.md   the findings behind every design choice
```

## Limits

- Needs the headset and SteamVR; the Touch controllers cannot be paired to a PC directly (they only bond to their headset).
- Tracking quality is the headset's: out of its camera view the controller falls back to gyro-only rotation.
- MotionPlus games (e.g. Wii Sports Resort) are untested: the bridge sends roll-only gyro data.
- The Air Link stream drains the headset battery faster than a PC's USB port charges it.
- Tested on Quest 2, Windows 11, Dolphin 2609b, Wii Sports.

## Development

- `cargo run --release -p bridge` runs one bridge; `cargo test --workspace` runs the tests.
- Lint as in pade. The rules live in plain Rust config (`[lints]` in each `Cargo.toml`, `clippy.toml`, `rustfmt.toml`, `deny.toml`), so cargo runs them directly: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, `cargo deny check`, `cargo machete`.
- Node is used only for the lint tooling: `pnpm install` once, then `pnpm lint` / `pnpm lint:fix` run all of the above plus Markdown/JSON/YAML formatting, and `avi12-lint rust-config` regenerates the rule files from the shared `rust-config-avi12` preset. Exemptions live in `avi12-rust.local.toml`, each with a reason; never `#[allow]`.
- No third-party crates: small dependencies (CRC-32, DLL loading, Win32 calls, JSON) are written in the repo.
- See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Support

[GitHub Sponsors](https://github.com/sponsors/avi12) or [PayPal](https://paypal.me/avi12).

## License

[Apache-2.0](LICENSE)
