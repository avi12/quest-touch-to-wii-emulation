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

use bridge::session::record::Record;
use numeric::format::{js_number, to_fixed};
use tools::cli::command_line;
use tools::telemetry::{
    TelemetryOptions, USAGE, axis_score_lines, diagnostics_line, elapsed_seconds, load_records,
    millis, row_lines, split_takes, summary_lines,
};

const TAKE_LENGTH_DECIMALS: usize = 1;
const AGE_DECIMALS: usize = 1;

fn print_lines(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn print_takes(records: &[Record], has_rows: bool) {
    let takes = split_takes(records);
    println!("{} recordings (B held)", takes.len());
    for (i_take, take) in takes.iter().enumerate() {
        let first = take.first().map_or(0.0, |record| record.timestamp_ms);
        let last = take.last().map_or(first, |record| record.timestamp_ms);
        let label = format!(
            "recording {}, {}s",
            i_take + 1,
            to_fixed(elapsed_seconds(first, last), TAKE_LENGTH_DECIMALS)
        );
        print_lines(&summary_lines(take, &label));
        println!("{}", diagnostics_line(take));
        print_lines(&axis_score_lines(take));
        if has_rows {
            print_lines(&row_lines(take, first));
        }
    }
}

fn now_ms() -> f64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    millis(elapsed)
}

fn print_recent(records: &[Record], newest_ms: f64, options: &TelemetryOptions) {
    let since = newest_ms - millis(options.window);
    let recent: Vec<&Record> = records
        .iter()
        .filter(|record| record.timestamp_ms >= since)
        .collect();
    let label = format!(
        "last {}s, age {}s",
        js_number(options.window.as_secs_f64()),
        to_fixed(elapsed_seconds(newest_ms, now_ms()), AGE_DECIMALS)
    );
    print_lines(&summary_lines(&recent, &label));
    if options.has_rows {
        print_lines(&row_lines(&recent, newest_ms));
    }
}

fn main() -> ExitCode {
    let options = match TelemetryOptions::parse(&command_line()) {
        Ok(options) => options,
        Err(error) => return error.report(USAGE),
    };
    let records = match load_records(&options.file) {
        Ok(records) => records,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let Some(newest) = records.last() else {
        eprintln!("no samples in {}", options.file.display());
        return ExitCode::FAILURE;
    };
    if options.has_marks {
        print_takes(&records, options.has_rows);
        return ExitCode::SUCCESS;
    }
    print_recent(&records, newest.timestamp_ms, &options);
    ExitCode::SUCCESS
}
