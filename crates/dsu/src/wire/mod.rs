//! Wire format: the bytes Dolphin sends and reads.

pub(crate) mod analog;
pub(crate) mod buttons;
mod crc;
mod float;
pub(crate) mod frame;
pub(crate) mod pad;
pub(crate) mod protocol;
