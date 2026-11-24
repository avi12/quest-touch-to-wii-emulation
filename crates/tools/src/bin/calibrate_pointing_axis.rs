//! Find the axis a controller rolls around ("twist it like a screwdriver"): a clean measurement of
//! how YOU hold it.
//!
//!     calibrate_pointing_axis
//!
//! Hold the controller aimed at the screen; after 10 s it records for 10 s while you twist it
//! around its pointing direction without changing where it points. Prints the axis in the
//! controller frame; add it to `CANDIDATES` in the bridge to use it.

use std::process::ExitCode;
use std::thread::sleep;
use std::time::{Duration, Instant};

use openvr_sys::{Mat3, OpenVr};
use tools::twist::TwistRecorder;

const WARMUP: Duration = Duration::from_secs(10);
const RECORD: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);

fn rotation(openvr: &OpenVr, hand: u32) -> Result<Mat3, String> {
    openvr
        .poses()
        .get(hand)
        .map(|pose| pose.rot)
        .ok_or_else(|| "controller pose unavailable".to_owned())
}

fn run() -> Result<String, String> {
    let openvr = tools::steamvr::open()?;
    let hand = tools::steamvr::right_controller(&openvr)?;
    sleep(WARMUP);
    println!("recording - twist now");
    let mut recorder = TwistRecorder::new(rotation(&openvr, hand)?);
    let end = Instant::now() + RECORD;
    while Instant::now() < end {
        sleep(POLL);
        recorder.observe(rotation(&openvr, hand)?);
    }
    recorder
        .axis()
        .map(|twist| twist.to_string())
        .ok_or_else(|| "no rotation recorded - is the controller tracked and moving?".to_owned())
}

fn main() -> ExitCode {
    match run() {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
