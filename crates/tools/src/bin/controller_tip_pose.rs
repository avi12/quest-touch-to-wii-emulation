//! Ask `SteamVR` for the Touch controller "tip" (aim ray) transform relative to its tracked pose.
//!
//! On a Quest 2 this prints a forward direction of about (0, -0.61, -0.79) in the controller
//! frame: `SteamVR`'s idea of where the controller "points". The bridge defaults to the controller
//! -z axis instead, because that matches how a Wiimote is held.

use std::process::ExitCode;

use numeric::format::rounded_vector;
use openvr_sys::{Component, ComponentTransform, StringProperty};

const NAME_COLUMN_WIDTH: usize = 9;
const DECIMALS: usize = 3;

fn format_component(name: &str, transform: &ComponentTransform) -> String {
    // Forward is -z: the negated third column of the rotation.
    let forward = transform.rot.map(|[_x, _y, z_column]| -z_column);
    format!(
        "{name:<NAME_COLUMN_WIDTH$} forward(-z) in controller frame: {} origin: {}",
        rounded_vector(&forward, DECIMALS),
        rounded_vector(&transform.origin, DECIMALS),
    )
}

fn run() -> Result<(), String> {
    let openvr = tools::steamvr::open()?;
    let hand = tools::steamvr::right_controller(&openvr)?;
    let model = openvr.string_property(hand, StringProperty::RenderModelName);
    println!("render model: {model}");
    for component in Component::ALL {
        let name = component.name().to_string_lossy();
        match openvr.component_transform(hand, &model, component) {
            Some(transform) => println!("{}", format_component(&name, &transform)),
            None => println!("{name:<NAME_COLUMN_WIDTH$} not available"),
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
