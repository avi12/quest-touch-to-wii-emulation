//! Double-click mode: with no `BRIDGE_HAND`, this exe starts both bridges as child processes (right
//! controller = Wii Remote 1, left = Wii Remote 2) and stops the other when one ends.
//!
//! `BRIDGE_PORT` on the launcher moves both: right gets it, left gets the next port. A
//! `BRIDGE_TELEMETRY` folder on the launcher gives each child `telemetry-<hand>.jsonl` in it.

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, ExitStatus};
use std::thread;
use std::time::Duration;

use dsu::Port;
use openvr_sys::Hand;

use crate::config::{HAND_VAR, PORT_VAR, TELEMETRY_VAR, hand_name};

/// How often the launcher checks whether a bridge has ended.
const POLL_EVERY: Duration = Duration::from_millis(200);

/// Whether this process is the launcher rather than one bridge.
pub fn is_launcher() -> bool {
    env::var_os(HAND_VAR).is_none()
}

/// Starts both bridges and waits; returns the exit code of the first one to end.
pub fn run() -> ExitCode {
    match run_both() {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("touch_dsu: could not start the bridges: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_both() -> io::Result<ExitStatus> {
    let right_port = base_port()?;
    let left_port = right_port
        .get()
        .checked_add(1)
        .and_then(Port::new)
        .ok_or_else(|| io::Error::other("no port above BRIDGE_PORT for the left bridge"))?;
    let mut right = spawn(Hand::Right, right_port)?;
    let mut left = match spawn(Hand::Left, left_port) {
        Ok(child) => child,
        Err(error) => {
            let _stopped = right.kill();
            return Err(error);
        }
    };
    println!(
        "touch_dsu: right controller on UDP {}, left on UDP {}; close this window to stop both",
        right_port.get(),
        left_port.get()
    );
    loop {
        if let Some(status) = first_to_end(&mut right, &mut left)? {
            return Ok(status);
        }
        thread::sleep(POLL_EVERY);
    }
}

/// The status of whichever child has ended, after stopping the other one.
fn first_to_end(first: &mut Child, second: &mut Child) -> io::Result<Option<ExitStatus>> {
    if let Some(status) = first.try_wait()? {
        let _stopped = second.kill();
        return Ok(Some(status));
    }
    if let Some(status) = second.try_wait()? {
        let _stopped = first.kill();
        return Ok(Some(status));
    }
    Ok(None)
}

fn base_port() -> io::Result<Port> {
    let Some(raw) = env::var_os(PORT_VAR) else {
        return Ok(Port::DEFAULT);
    };
    raw.to_str()
        .and_then(|text| text.trim().parse::<u16>().ok())
        .and_then(Port::new)
        .ok_or_else(|| io::Error::other("BRIDGE_PORT must be an integer from 1 to 65535"))
}

fn spawn(hand: Hand, port: Port) -> io::Result<Child> {
    let name = hand_name(hand);
    let mut command = Command::new(env::current_exe()?);
    command
        .env(HAND_VAR, name)
        .env(PORT_VAR, port.get().to_string());
    if let Some(folder) = env::var_os(TELEMETRY_VAR) {
        command.env(
            TELEMETRY_VAR,
            PathBuf::from(folder).join(format!("telemetry-{name}.jsonl")),
        );
    }
    command.spawn()
}

fn exit_code(status: ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}
