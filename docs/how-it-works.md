# How it works

Every design choice below came from an experiment or from Dolphin's source. Facts marked _(source)_ were read from
[Dolphin's source](https://github.com/dolphin-emu/dolphin) (`DualShockUDPClient.cpp`, `DualShockUDPProto.h`, `WiimoteEmu.cpp`).

## 1. Reading the controllers

| Finding                                                                                                                  | Consequence                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| Touch controllers only bond to their own headset; a PC Bluetooth adapter never sees them, not even in pairing mode       | No direct-to-PC path. The headset must be connected and tracking.                                                             |
| Meta's runtime (`LibOVRRT64_1.dll`) refuses a third-party process (`ovr_Initialize` → `-3002`)                           | Read poses through **SteamVR / OpenVR** instead.                                                                              |
| Over Air Link the controllers appear to SteamVR as ordinary tracked controllers                                          | The bridge uses `VRApplication_Background`; nothing is rendered.                                                              |
| Steam Link (`vrlink` driver) and Virtual Desktop's driver fight Meta's Link for the headset                              | Run only one streaming path; close the other streamer.                                                                        |
| Meta Link is greyed out in the headset until the PC app is signed in and has the headset registered                      | Sign in on both with the same account; Air Link works without the cable.                                                      |
| A sleeping headset (proximity sensor) drops the controllers                                                              | `adb shell am broadcast -a com.oculus.vrpowermanager.prox_close` every 20 s keeps it "worn" (`tools/keep_headset_awake.ps1`). |
| Outside the headset cameras' view, `eTrackingResult` stays 200 but **position freezes**; rotation continues from the IMU | Aim only needs rotation, but yaw drifts without camera help. Hence the recenter and the placement advice.                     |

## 2. The DSU pad Dolphin sees

_(source)_ Dolphin's DSU client maps these fields, which differ from the protocol spec in places:

| Dolphin input                                                            | Comes from                | Notes                                                                                                               |
| ------------------------------------------------------------------------ | ------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `Cross` `Circle` `Square` `Triangle`, `Pad N/S/E/W`, `L1` `R1` `L2` `R2` | **analog** bytes          | The digital button bits are ignored for these. Field order is Square, Cross, Circle, Triangle, then R1, L1, R2, L2. |
| `Share` `Options` `L3` `R3` `PS`                                         | digital bits              |                                                                                                                     |
| `Left/Right X±`, `Y±`                                                    | stick bytes, 128 = centre | The Y fields are "inverted": 255 = up.                                                                              |
| `Accel Up/Down/Left/Right/Forward/Backward`                              | accel x, y, z in g        | x = −right, y = −up, z = forward. Resting still, level: y = −1.                                                     |
| `Gyro Pitch/Yaw/Roll …`                                                  | gyro deg/s                | +pitch = nose up, +yaw = right, +roll = clockwise from behind.                                                      |

Clients are dropped after 5 s without a pad request, so a restarted bridge is picked up again automatically.

## 3. Controller frame → Wiimote frame

The Touch pose frame is not the Wiimote's (x right, y forward, z up). The bridge builds the Wiimote axes from a **pointing axis** `F0`
in the controller frame (default: the controller's −z, picked live with the selector for a Wiimote-style grip) and an "up" taken from gravity at the last recenter.
Candidates (SteamVR's own `tip` ray is (0, −0.61, −0.79) on a Quest 2, `controller_tip_pose` (crates/tools)) can be cycled live with Ctrl+Alt+[ ].
How _you_ hold the controller decides which axis feels right; `calibrate_pointing_axis` (crates/tools) measures a twist axis, but a twist and the aim differ for some grips.

## 4. The pointer

Dolphin builds the camera transform from **two stacked sources** _(source: `GetTotalTransformation`)_: the IR/"Point" group and the IMU "Point" group
(on by default). The bridge uses them for two different jobs:

| Job                     | Mechanism                                                                                                                                                                                                                                                                                                  |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Where the hand is       | The bridge intersects the aim ray with a flat virtual screen defined at the last recenter and sends the position on the **right-stick axes**, mapped in Dolphin to `IR/Up…Right`. ±1 = screen edge, 0,0 = centre (verified by driving known values and reading the screen).                                |
| How the hand is tilted  | Dolphin's IMU pointer turns the hand by the Wiimote roll. The bridge feeds it **roll only**: gravity rotated by the roll about the aim axis, and only the roll rate.                                                                                                                                       |
| Keeping the two centred | `IMUIR/Recenter = PS`; the bridge holds the PS button for 0.3 s at every recenter, through every swing and for 1 s after it, until Dolphin has eased its tilt back (2% per update by default). Dolphin tilts its motion pointer toward the accelerometer each frame (`EmulateIMUCursor`, `Dynamics.cpp`), so a fast sideways swing would otherwise tip the cursor up or down. |

Details that matter:

- The pointer is `tan`-scaled (flat screen), so a pitched grip doesn't tilt the horizontal sweep. Angle-based mapping did.
- A One-Euro filter smooths it (jitter when slow, no lag when fast). Range: ±28° across, ±30° up/down.
- Auto-recenter when the pointer is pinned at an edge while the controller is held still for 0.8 s (a shake alone never moves the screen; never when pointing straight up or down).
- The pointer uses the full ±1: Dolphin's default IR settings map ±1 to the screen edges (Wii Menu included), and its geometry keeps both IR dots inside the 1024×768 camera frame there. An earlier ±0.95 cap left corner buttons (the Wii Menu's Continue) out of reach. Roll rate is not sent during a swing (Dolphin would integrate it into a lasting tilt).
- When the aim points away from the screen (more than 60° off its normal) there is no hit point, so the pointer keeps its pre-swing position instead of jumping to a corner.
- While a swing is in progress (acceleration over 1.5 g) the pointer still follows the aim, so fast pointing works; only when the swing sweeps it to the screen edge does it keep its last on-screen position, until 200 ms after the swing ends.
- Swing acceleration is passed to Dolphin only above ~0.6–1.2 g. Gentle aiming accelerations bend the IMU pointer's _pitch_ (Dolphin corrects pitch with the accelerometer), which made vertical motion wobble.

## 5. Dolphin gotchas

- **Edit config only while Dolphin is closed**; it rewrites the files on exit. It also drops default-valued keys (`IMUIR/Enabled = True` disappears).
- F10 = Pause, F9 = Screenshot in Dolphin. Hotkeys that clash freeze or spam it.
- **Background Input** (`Dolphin.ini`, `[Input]`, `BackgroundInput = True`) lets the game keep controller input without focus.
- Wii Sports asks to unplug the extension if an emulated Nunchuk is attached; set **Extension: None** for the remote.
- A second remote needs its own DSU server entry (device `DSUClient/0/<description>`), `Source = 1` in `[Wiimote2]`, and Wii Remote 2 set to Emulated.

## 6. Diagnosing with screenshots

The bridge has a `PTR_TEST=1` mode that sweeps the pointer through (0,0), (±1,0), (0,±1) every 4 s while sending a level, still accelerometer.
Photographing Dolphin's window in each slot reveals the real mapping. This is how the half-screen offset caused by the stacked IMU pointer was found.
