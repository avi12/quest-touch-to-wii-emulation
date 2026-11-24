//! Quest Touch controllers -> Dolphin Wii Remotes, as DSU (Cemuhook) pads.
//!
//! Reads the Touch controllers through `SteamVR` (`OpenVR`) and serves one DSU pad on UDP 26760.
//! The right controller is Wii Remote 1: aimed pointer, roll, swings, buttons; the left stick
//! drives a Nunchuk. A second copy (`BRIDGE_PORT=26761 BRIDGE_HAND=left`) turns the left
//! controller into Wii Remote 2.
//!
//! Needs `SteamVR` running with the headset connected (Air Link / Link / Virtual Desktop). No VR
//! display is used. Hotkeys (global, Ctrl+Alt so they never clash with Dolphin's): L toggles the
//! left controller, ] and [ cycle the pointing axis. Thumbstick click = recenter. See README.md
//! and docs/how-it-works.md.

use std::process::ExitCode;

use bridge::config::Config;
use bridge::runtime::run_bridge;

fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("touch_dsu: invalid setting: {error}");
            return ExitCode::FAILURE;
        }
    };
    let Err(error) = run_bridge(config);
    eprintln!("touch_dsu: {error}");
    ExitCode::FAILURE
}
