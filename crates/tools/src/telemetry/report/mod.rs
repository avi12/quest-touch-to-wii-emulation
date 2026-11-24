//! The printed lines: summary, findings, pointing-axis scores and 250 ms rows.

mod axes;
mod findings;
mod rows;
mod series;
mod summary;

pub use axes::axis_score_lines;
pub use findings::diagnostics_line;
pub use rows::row_lines;
pub use summary::summary_lines;
