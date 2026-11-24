//! List what `SteamVR` currently tracks: device class, controller role, pose validity, tracking
//! result, position.
//!
//! A controller that `SteamVR` cannot see shows up as 'valid 0' or is missing; 'pos' frozen while
//! it moves means the headset cameras have lost it (rotation still updates from the controller
//! IMU).

use std::process::ExitCode;

use numeric::format::rounded;
use openvr_sys::DeviceIndex;

const POSITION_DECIMALS: usize = 2;

/// Rounded to two decimals, always with a fraction digit (`1.0`, `-0.25`).
fn format_coordinate(value: f64) -> String {
    let text = rounded(value, POSITION_DECIMALS);
    if text.contains('.') {
        return text;
    }
    format!("{text}.0")
}

/// The variant's name, or `unknown` for a code the binding does not know.
fn debug_name(value: Option<impl std::fmt::Debug>) -> String {
    value.map_or_else(|| "unknown".to_owned(), |known| format!("{known:?}"))
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
    for index in DeviceIndex::all().filter(|index| openvr.is_connected(*index)) {
        let pose = poses.get(index);
        let position: Vec<String> = pose
            .pos
            .iter()
            .map(|value| format_coordinate(*value))
            .collect();
        println!(
            "{} class {} role {} valid {} result {} pos [{}]",
            index.get(),
            debug_name(openvr.device_class(index)),
            debug_name(openvr.controller_role(index)),
            u8::from(pose.is_valid),
            debug_name(pose.tracking),
            position.join(", "),
        );
    }
    ExitCode::SUCCESS
}
