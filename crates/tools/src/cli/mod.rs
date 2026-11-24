//! Small typed command-line parsing: positional arguments, `--flags`, validated values.

mod error;
mod parse;
mod port;

pub use error::{ArgError, EXIT_BAD_USAGE};
pub use parse::{command_line, has_flag, positional, positive_duration};
pub use port::parse_port;
