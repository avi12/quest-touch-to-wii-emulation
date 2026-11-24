//! One telemetry line, as the bridge writes it (JSON Lines).

use std::fmt;
use std::io;

use serde::Deserialize;

/// One bridge loop sample. Lines that do not fit this shape are skipped.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    /// Milliseconds since the Unix epoch.
    pub timestamp: f64,
    /// Aim pitch, degrees above level.
    pub pitch: f64,
    /// Aim yaw, degrees.
    pub yaw: f64,
    /// Pointer, -1..1 across the screen.
    pub pointer_x: f64,
    pub pointer_y: f64,
    /// Acceleration sent to Dolphin (g).
    pub accel: [f64; 3],
    /// Roll rate sent to Dolphin (deg/s).
    pub gyro_roll: f64,
    /// Controller-to-world rotation, row-major 3x3.
    pub rotation: [f64; 9],
    /// Cosine between the aim and the screen normal (absent in older logs).
    #[serde(default = "default_facing")]
    pub facing: f64,
    pub recenter: bool,
    /// The right controller's B button was held: part of a recording.
    pub mark: bool,
}

const fn default_facing() -> f64 {
    1.0
}

/// The telemetry file could not be read.
#[derive(Debug)]
pub struct LoadError {
    pub file: String,
    pub source: io::Error,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot read {}: {}", self.file, self.source)
    }
}

impl std::error::Error for LoadError {}

/// Every well-formed sample of `file`, in order.
pub fn load_samples(file: &str) -> Result<Vec<Sample>, LoadError> {
    let text = std::fs::read_to_string(file).map_err(|source| LoadError {
        file: file.to_owned(),
        source,
    })?;
    Ok(parse_samples(&text))
}

fn parse_samples(text: &str) -> Vec<Sample> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"timestamp":1,"pitch":-20.78,"yaw":-85.9,"pointerX":0,"pointerY":0,"accel":[0.01,-1,0],"gyroRoll":78.75,"rotation":[1,0,0,0,1,0,0,0,1],"recenter":true,"mark":false,"extra":3}"#;

    #[test]
    fn parses_valid_lines_and_skips_the_rest() {
        let text = format!("{LINE}\n\nnot json\n{{\"timestamp\":2}}\n{LINE}\n");
        let samples = parse_samples(&text);
        assert_eq!(samples.len(), 2);
        assert!((samples[0].facing - 1.0).abs() < f64::EPSILON);
        assert!((samples[0].pitch + 20.78).abs() < f64::EPSILON);
        assert!(samples[0].recenter);
    }
}
