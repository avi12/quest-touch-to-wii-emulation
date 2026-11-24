//! Summaries of a bridge telemetry stream (`BRIDGE_TELEMETRY`, `scripts\start.ps1 -Telemetry`).

mod options;
mod report;
mod sample;
mod takes;

pub use options::{TelemetryOptions, USAGE};
pub use report::{axis_score_lines, diagnostics_line, row_lines, summary_lines};
pub use sample::{LoadError, Sample, load_samples};
pub use takes::split_takes;

/// Telemetry timestamps are milliseconds.
pub const MS_PER_SECOND: f64 = 1000.0;
