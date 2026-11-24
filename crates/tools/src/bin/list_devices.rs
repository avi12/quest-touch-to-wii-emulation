//! List what `SteamVR` currently tracks: device class, controller role, pose validity, tracking
//! result, position.
//!
//! A controller that `SteamVR` cannot see shows up as 'valid 0' or is missing; 'pos' frozen while
//! it moves means the headset cameras have lost it (rotation still updates from the controller
//! IMU).

use std::process::ExitCode;

use openvr_sys::MAX_DEVICES;
use tools::format::rounded;

const POSITION_DECIMALS: usize = 2;

/// Rounded to two decimals, always with a fraction digit (`1.0`, `-0.25`).
fn format_coordinate(value: f64) -> String {
    let text = rounded(value, POSITION_DECIMALS);
    if text.contains('.') {
        return text;
    }
    format!("{text}.0")
}

fn main() -> ExitCode {
    let openvr = match tools::steamvr::open() {
        Ok(openvr) => openvr,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let poses = openvr.poses();
    let indices = (0..MAX_DEVICES).filter_map(|index| u32::try_from(index).ok());
    for index in indices.filter(|index| openvr.is_connected(*index)) {
        let Some(pose) = poses.get(index) else {
            continue;
        };
        let position: Vec<String> = pose
            .pos
            .iter()
            .map(|value| format_coordinate(*value))
            .collect();
        println!(
            "{index} class {} role {} valid {} result {} pos [{}]",
            openvr.device_class(index),
            openvr.controller_role(index),
            u8::from(pose.valid),
            pose.tracking_result,
            position.join(", "),
        );
    }
    ExitCode::SUCCESS
}
