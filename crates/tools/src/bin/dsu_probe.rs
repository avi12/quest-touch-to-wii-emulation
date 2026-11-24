//! Connect to a running bridge as a DSU client and print what Dolphin would see.
//!
//!     dsu_probe [port] [seconds]
//!
//! Prints the min/max of the pointer axes (128 = centre), the largest gyro rate and the pressed
//! buttons. Use it to check the bridge without Dolphin: move the controller while it runs.

use std::process::ExitCode;
use std::time::{Duration, Instant};

use tools::args::command_line;
use tools::probe::{DsuClient, LOOPBACK, PadReading, ProbeOptions, ProbeSummary, USAGE};

fn main() -> ExitCode {
    let options = match ProbeOptions::parse(&command_line()) {
        Ok(options) => options,
        Err(error) => return error.report(USAGE),
    };
    let client = match DsuClient::new(options.port) {
        Ok(client) => client,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut summary = ProbeSummary::default();
    let end = Instant::now() + Duration::from_secs_f64(options.seconds);
    while Instant::now() < end {
        let reading = client
            .request_pad()
            .and_then(|data| PadReading::parse(&data));
        if let Some(reading) = reading {
            summary.add(&reading);
        }
    }
    if summary.packets() == 0 {
        eprintln!(
            "no pad data from {LOOPBACK}:{} - is the bridge running and the controller tracked?",
            options.port
        );
        return ExitCode::FAILURE;
    }
    println!("{summary}");
    ExitCode::SUCCESS
}
