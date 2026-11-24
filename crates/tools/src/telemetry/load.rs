//! Reads a telemetry file: every line the bridge's [`Record`] schema accepts.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use bridge::session::record::Record;

/// The telemetry file could not be read.
#[derive(Debug)]
pub struct LoadError {
    pub file: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot read {}: {}",
            self.file.display(),
            self.source
        )
    }
}

impl std::error::Error for LoadError {}

/// Every well-formed record of `file`, in order; other lines are skipped.
pub fn load_records(file: &Path) -> Result<Vec<Record>, LoadError> {
    let text = std::fs::read_to_string(file).map_err(|source| LoadError {
        file: file.to_owned(),
        source,
    })?;
    Ok(parse_records(&text))
}

fn parse_records(text: &str) -> Vec<Record> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(Record::parse_line)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::telemetry::load::parse_records;

    const LINE: &str = r#"{"timestamp":1,"pitch":-20.78,"yaw":-85.9,"pointerX":0,"pointerY":0,"accel":[0.01,-1,0],"gyroRoll":78.75,"rotation":[1,0,0,0,1,0,0,0,1],"recenter":true,"mark":false,"extra":3}"#;

    #[test]
    fn keeps_valid_lines_and_skips_the_rest() {
        let text = format!("{LINE}\n\nnot json\n{{\"timestamp\":2}}\n{LINE}\n");
        let records = parse_records(&text);
        assert_eq!(records.len(), 2);
        let first = records.first();
        assert!(first.is_some_and(|record| record.has_recentered));
        assert!(first.is_some_and(|record| (record.facing - 1.0).abs() < f64::EPSILON));
        assert!(first.is_some_and(|record| (record.pitch + 20.78).abs() < f64::EPSILON));
    }
}
