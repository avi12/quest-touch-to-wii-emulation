# CLAUDE.md

Bridge that lets Meta Quest Touch controllers act as Dolphin Wii Remotes. Read `README.md` for use and `docs/how-it-works.md` for the findings behind each choice.

## Map

- `touch_dsu.ts`: thin entry point (Node 26 runs `.ts` natively). The bridge lives in `src/`: `loop.ts` (250 Hz loop), `motion.ts` (pose → accel / gyro / swing), `pointer.ts` (recenter + pointer), `buttons.ts`, `hotkeys.ts`, `tracking.ts`, `config.ts` (zod-validated env), `math.ts`, `openvr.ts` (koffi binding to SteamVR), `dsu_server.ts` (DSU / Cemuhook protocol, no SteamVR).
- `scripts/start.ps1`, `stop.ps1`: run both bridges; PID file based.
- `config/`: Dolphin config for Wii Remote 1 and 2 (must be merged into Dolphin's config dir).
- `tools/`: diagnostics (`.ts`, run with `node tools/<name>.ts`), no side effects (except `keep_headset_awake.ps1`, which changes headset settings until `-Restore`).

## Rules

- **Never kill processes by name** (`taskkill /IM node.exe`, `Stop-Process -Name …`). The user has other Node, Python, Dolphin and VR apps running. Use `scripts/stop.ps1` or a PID you started.
- **Dolphin must be closed before editing its config** (`%APPDATA%\Dolphin Emulator\Config\*.ini`); it overwrites them on exit. Back the file up first (`*.bak`), tell the user to close it, check with `tasklist | grep -ic dolphin`, then edit.
- **Do not use Dolphin's default hotkeys** for bridge features: F10 = Pause, F9 = Screenshot, plus F1–F12 others. Bridge hotkeys are Ctrl+Alt+key.
- Changes to the bridge need no Dolphin restart; changes to the Dolphin config do (after closing it).
- Do not commit personal data: account names, device serials, IP addresses, `%APPDATA%` dumps, logs.
- Verify changes in the real app, not by reasoning: drive `PTR_TEST=1` and screenshot Dolphin, or run `node tools/dsu_probe.ts` while moving the controller.

## Working on the bridge

- Setup: Node 26 + `pnpm install`. Run: `pnpm start` (right controller → port 26760) and `BRIDGE_PORT=26761 BRIDGE_HAND=left pnpm start`. Needs SteamVR up with the headset connected, or `openvr.init` fails (`InitError_Driver_WirelessHmdNotConnected` = headset not streaming yet).
- Dolphin's DSU conventions are not the spec's; they come from its source (see how-it-works §2). If a button or axis is wrong, check there before guessing: the face buttons and D-pad are **analog** fields, the stick Y is 255 = up, accel is x = −right, y = −up, z = forward.
- The pointer and the hand roll use **separate Dolphin paths** (IR group vs IMU group). Don't feed real yaw/pitch to the IMU group: it stacks on the aim and shifts the cursor by half a screen. Don't pass gentle accelerations either: they wobble its pitch.
- The bridge's tunables are constants (`config.ts` / top of `touch_dsu.ts`) (`YAW_RANGE`, `PITCH_RANGE`, `SWING_LO/HI`, `ROLL_GAIN`, `F0`).
- Quest quirks: a sleeping headset drops the controllers (`tools/keep_headset_awake.ps1`); out of the headset cameras' view the position freezes but rotation still updates; Git Bash rewrites `/sdcard/...` arguments to `adb`, so use PowerShell for adb file paths.

## Conventions

- Lint = `pnpm lint` (`pnpm lint:fix` fixes; `pnpm check` types; `pnpm lint:all` adds fallow). Naming rules follow the youtube-time-manager style via `oxlint-plugin-naming`.

- `main` is the only long-lived branch.
- One logical change per commit, message says what and why. Keep docs in step with behaviour changes (README controls table, how-it-works).
