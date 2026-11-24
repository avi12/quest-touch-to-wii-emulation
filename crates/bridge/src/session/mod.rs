//! What happens around the pad stream: the second controller coming and going, and the optional
//! telemetry log.

mod telemetry;
mod tracking;

pub use telemetry::{Telemetry, TelemetrySample, telemetry_line};
pub use tracking::{SecondControllerWatcher, beep_switch};
