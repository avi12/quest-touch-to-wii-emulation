//! String properties of tracked devices, such as the render-model name.

use crate::device::code::c_enum;
use crate::device::index::DeviceIndex;
use crate::ffi::table::{SystemTable, len_u32};

const PROPERTY_BUFFER_BYTES: usize = 256;

c_enum! {
    /// An `ETrackedDeviceProperty` whose value is a string.
    StringProperty {
        /// `Prop_RenderModelName_String`.
        RenderModelName = 1003,
    }
}

/// The property's value, or "" when the device has none.
pub(crate) fn read(system: &SystemTable, index: DeviceIndex, property: StringProperty) -> String {
    let mut buffer = [0_u8; PROPERTY_BUFFER_BYTES];
    let mut error = 0;
    // SAFETY: `buffer` is writable for the size passed, and `error` is a valid out-pointer.
    let length = unsafe {
        (system.string_property)(
            index.get(),
            property.code(),
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
    use crate::device::property::decode_text;

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
