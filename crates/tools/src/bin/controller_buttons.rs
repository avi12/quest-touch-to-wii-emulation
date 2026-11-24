//! `controller_buttons [seconds]`: prints what `SteamVR` reports for each controller's buttons,
//! sticks and triggers whenever it changes, to tell a dead button from a bridge mapping problem.

use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use openvr_sys::{Button, ControllerState, Hand, OpenVr};
use tools::cli::{EXIT_BAD_USAGE, command_line, positional, positive_duration};
use tools::steamvr;

const DEFAULT_DURATION: Duration = Duration::from_secs(20);
const POLL_EVERY: Duration = Duration::from_millis(20);
const BUTTONS: [Button; 4] = [
    Button::Primary,
    Button::Secondary,
    Button::Click,
    Button::System,
];

fn describe(hand: Hand, state: Option<&ControllerState>) -> String {
    let Some(state) = state else {
        return format!("{hand:?}: no state (SteamVR returned nothing)");
    };
    let held: Vec<String> = BUTTONS
        .iter()
        .filter(|button| state.is_down(**button))
        .map(|button| format!("{button:?}"))
        .collect();
    format!(
        "{hand:?}: held [{}] stick ({:.2}, {:.2}) trigger {:.2} grip {:.2}",
        held.join(" "),
        state.stick[0],
        state.stick[1],
        state.trig,
        state.grip
    )
}

fn snapshot(openvr: &OpenVr) -> String {
    [Hand::Right, Hand::Left]
        .into_iter()
        .map(|hand| match openvr.controller_index(hand) {
            Some(index) => describe(hand, openvr.controller_state(index).as_ref()),
            None => format!("{hand:?}: not connected"),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn main() -> ExitCode {
    let arguments = command_line();
    let values = positional(&arguments);
    let duration = match positive_duration(values.first().copied(), "seconds", DEFAULT_DURATION) {
        Ok(duration) => duration,
        Err(error) => {
            eprintln!("usage: controller_buttons [seconds]\n{error}");
            return ExitCode::from(EXIT_BAD_USAGE);
        }
    };
    let openvr = match steamvr::open() {
        Ok(openvr) => openvr,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let start = Instant::now();
    let mut last = String::new();
    while start.elapsed() < duration {
        let current = snapshot(&openvr);
        if current != last {
            println!("{:>5.2}s {current}", start.elapsed().as_secs_f64());
            last = current;
        }
        thread::sleep(POLL_EVERY);
    }
    ExitCode::SUCCESS
}
