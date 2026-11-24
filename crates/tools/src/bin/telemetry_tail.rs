//! Summarise a bridge telemetry stream (see `BRIDGE_TELEMETRY` / `scripts\start.ps1 -Telemetry`).
//!
//!     telemetry_tail [file] [seconds] [--rows] [--marks]
//!
//! Prints min / max / spread of the aim angles, pointer and acceleration, how long the pointer sat
//! at an edge, and recentres, for the last <seconds>. With --rows it also prints one row per
//! 250 ms. Holding the right controller's B button records a take: with --marks each held stretch
//! is analysed as one recording.

use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use tools::args::command_line;
use tools::format::{js_number, to_fixed};
use tools::telemetry::{
    MS_PER_SECOND, Sample, TelemetryOptions, USAGE, axis_score_lines, diagnostics_line,
    load_samples, row_lines, split_takes, summary_lines,
};

fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn print_takes(samples: &[Sample], rows: bool) {
    let takes = split_takes(samples);
    println!("{} recordings (B held)", takes.len());
    for (index, take) in takes.iter().enumerate() {
        let first = take.first().map_or(0.0, |sample| sample.timestamp);
        let last = take.last().map_or(first, |sample| sample.timestamp);
        let label = format!(
            "recording {}, {}s",
            index + 1,
            to_fixed((last - first) / MS_PER_SECOND, 1)
        );
        print_lines(&summary_lines(take, &label));
        println!("{}", diagnostics_line(take));
        print_lines(&axis_score_lines(take));
        if rows {
            print_lines(&row_lines(take, first));
        }
    }
}

fn now_ms() -> f64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    elapsed.as_secs_f64() * MS_PER_SECOND
}

fn print_recent(samples: &[Sample], newest: f64, options: &TelemetryOptions) {
    let since = newest - options.seconds * MS_PER_SECOND;
    let recent: Vec<Sample> = samples
        .iter()
        .filter(|sample| sample.timestamp >= since)
        .copied()
        .collect();
    let label = format!(
        "last {}s, age {}s",
        js_number(options.seconds),
        to_fixed((now_ms() - newest) / MS_PER_SECOND, 1)
    );
    print_lines(&summary_lines(&recent, &label));
    if options.rows {
        print_lines(&row_lines(&recent, newest));
    }
}

fn main() -> ExitCode {
    let options = match TelemetryOptions::parse(&command_line()) {
        Ok(options) => options,
        Err(error) => return error.report(USAGE),
    };
    let samples = match load_samples(&options.file) {
        Ok(samples) => samples,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let Some(newest) = samples.last() else {
        eprintln!("no samples in {}", options.file);
        return ExitCode::FAILURE;
    };
    if options.marks {
        print_takes(&samples, options.rows);
        return ExitCode::SUCCESS;
    }
    print_recent(&samples, newest.timestamp, &options);
    ExitCode::SUCCESS
}
