# CLAUDE.md

Bridge that lets Meta Quest Touch controllers act as Dolphin Wii Remotes. Read `README.md` for use and `docs/how-it-works.md` for the findings behind each choice.

## Map

- Rust workspace, no third-party crates (supply-chain policy: write small dependencies ourselves).
  - `crates/bridge`: binary `touch_dsu`. `runtime/` (250 Hz loop), `motion/` (pose → accel / gyro / swing), `pointer/` (recenter + pointer), `input/` (buttons, hotkeys), `session/` (second-controller tracking, telemetry record = the JSON-lines format, shared with the tools), `config.rs` (validated env), `math/`.
  - `crates/dsu`: DSU / Cemuhook protocol (`wire/` bytes, `server/` socket), no SteamVR.
  - `crates/openvr-sys`, `crates/win32-sys`: the only crates with `unsafe` (FFI to `openvr_api.dll`, user32/kernel32/winmm).
  - `crates/geometry` (`Vec3`, `Mat3`), `crates/numeric` (the one checked float→int cast; the only crate allowed `as`).
  - `crates/tools`: diagnostics, one bin each.
- `scripts/start.ps1`, `stop.ps1`: run both bridges; PID file based.
- `config/`: Dolphin config for Wii Remote 1 and 2 (must be merged into Dolphin's config dir).
- Diagnostics (`cargo run -p tools --bin <name>`) have no side effects; `tools/keep_headset_awake.ps1` changes headset settings until `-Restore`.

## Rules

- **Never kill processes by name** (`taskkill /IM touch_dsu.exe`, `Stop-Process -Name …`). The user has other Node, Python, Dolphin and VR apps running. Use `scripts/stop.ps1` or a PID you started.
- **Dolphin must be closed before editing its config** (`%APPDATA%\Dolphin Emulator\Config\*.ini`); it overwrites them on exit. Back the file up first (`*.bak`), tell the user to close it, check with `tasklist | grep -ic dolphin`, then edit.
- **Do not use Dolphin's default hotkeys** for bridge features: F10 = Pause, F9 = Screenshot, plus F1–F12 others. Bridge hotkeys are Ctrl+Alt+key.
- Changes to the bridge need no Dolphin restart; changes to the Dolphin config do (after closing it).
- Do not commit personal data: account names, device serials, IP addresses, `%APPDATA%` dumps, logs.
- Verify changes in the real app, not by reasoning: drive `PTR_TEST=1` and screenshot Dolphin, or run `cargo run -p tools --bin dsu_probe` while moving the controller.

## Working on the bridge

- Setup: rustup (stable) + `pnpm install` (lint only). Run: `scripts\start.ps1` (builds release, starts right → 26760 and left → 26761), or one bridge with `cargo run --release -p bridge` (`BRIDGE_PORT=26761 BRIDGE_HAND=left` for the left). Needs SteamVR up with the headset connected, or OpenVR init fails (`InitError_Driver_WirelessHmdNotConnected` = headset not streaming yet).
- Dolphin's DSU conventions are not the spec's; they come from its source (see how-it-works §2). If a button or axis is wrong, check there before guessing: the face buttons and D-pad are **analog** fields, the stick Y is 255 = up, accel is x = −right, y = −up, z = forward.
- The pointer and the hand roll use **separate Dolphin paths** (IR group vs IMU group). Don't feed real yaw/pitch to the IMU group: it stacks on the aim and shifts the cursor by half a screen. Don't pass gentle accelerations either: they wobble its pitch.
- The bridge's tunables are constants (top of each module in `crates/bridge/src`, e.g. `YAW_RANGE`, `PITCH_RANGE`, `SWING_GAIN`, `MAX_SWING_G`).
- Quest quirks: a sleeping headset drops the controllers (`tools/keep_headset_awake.ps1`); out of the headset cameras' view the position freezes but rotation still updates; Git Bash rewrites `/sdcard/...` arguments to `adb`, so use PowerShell for adb file paths.

## Conventions

- Lint = `pnpm lint` (`pnpm lint:fix` fixes): avi12-lint as in pade - clippy pedantic (no `as`, unwrap, indexing, magic numbers), rustfmt, cargo-deny, cargo-machete, ast-grep house rules. Never `#[allow]`; lint levels are generated into each `Cargo.toml` by `avi12-lint rust-config`, exemptions go in `avi12-rust.local.toml` with a reason.
- Strong types: newtypes/enums for indices, units and roles; reuse `geometry`, `numeric` and existing helpers instead of re-deriving.

- `main` is the only long-lived branch.
- One logical change per commit, message says what and why. Keep docs in step with behaviour changes (README controls table, how-it-works).
