//! What happens around the pad stream: the second controller coming and going, and the optional
//! telemetry log.

mod json;
pub mod record;
mod telemetry;
mod tracking;

pub use telemetry::{Telemetry, TelemetrySample};
pub use tracking::{SecondControllerWatcher, beep_switch};
