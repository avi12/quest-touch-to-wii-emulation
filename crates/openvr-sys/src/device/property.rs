//! String properties of tracked devices, such as the render-model name.

use crate::ffi::table::{SystemTable, len_u32};

const PROPERTY_BUFFER_BYTES: usize = 256;

/// An `ETrackedDeviceProperty` whose value is a string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringProperty(pub i32);

/// The string properties the bridge reads.
#[derive(Clone, Copy, Debug)]
pub struct Properties {
    /// `Prop_RenderModelName_String`.
    pub render_model_name: StringProperty,
}

/// Device property ids, as in `SteamVR`'s `ETrackedDeviceProperty`.
pub const PROPERTY: Properties = Properties {
    render_model_name: StringProperty(1003),
};

/// The property's value, or "" when the device has none.
pub(crate) fn read(system: &SystemTable, index: u32, property: StringProperty) -> String {
    let mut buffer = [0_u8; PROPERTY_BUFFER_BYTES];
    let mut error = 0;
    // SAFETY: `buffer` is writable for the size passed, and `error` is a valid out-pointer.
    let length = unsafe {
        (system.string_property)(
            index,
            property.0,
            buffer.as_mut_ptr().cast(),
            len_u32(buffer.len()),
            &raw mut error,
        )
    };
    decode_text(&buffer, length)
}

/// The text before the terminating NUL that `length` counts.
fn decode_text(buffer: &[u8], length: u32) -> String {
    let text_len = usize::try_from(length.saturating_sub(1)).unwrap_or(0);
    buffer
        .get(..text_len)
        .map(|text| String::from_utf8_lossy(text).into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_the_terminating_nul() {
        assert_eq!(decode_text(b"oculus_quest\0junk", 13), "oculus_quest");
    }

    #[test]
    fn empty_when_nothing_was_written() {
        assert_eq!(decode_text(b"\0", 1), "");
        assert_eq!(decode_text(b"", 0), "");
    }
}
