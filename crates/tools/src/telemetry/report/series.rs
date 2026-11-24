//! One field across a window of records.

use bridge::session::record::Record;

use crate::stats::spread_text;

/// `min..max sd deviation` of `field` over `window`.
pub(crate) fn series(window: &[&Record], field: fn(&Record) -> f64) -> String {
    spread_text(
        &window
            .iter()
            .map(|record| field(record))
            .collect::<Vec<_>>(),
    )
}

/// How many records of `window` match `predicate`.
pub(crate) fn count(window: &[&Record], predicate: impl Fn(&Record) -> bool) -> usize {
    window.iter().filter(|record| predicate(record)).count()
}
