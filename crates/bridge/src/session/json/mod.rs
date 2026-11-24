//! Just enough JSON for the telemetry lines: one flat object per line, with numbers, booleans,
//! `null` and arrays of numbers as values. No third-party crate, so the format is all here.

mod reader;
mod writer;

pub(crate) use reader::{JsonValue, parse_flat_object};
pub(crate) use writer::JsonObject;

#[cfg(test)]
mod tests;
