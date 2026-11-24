//! Tracked-device indices: always a slot `SteamVR` reports.

/// How many tracked devices `OpenVR` reports (`k_unMaxTrackedDeviceCount`).
pub(crate) const MAX_DEVICES: usize = 64;

/// One tracked-device slot, always below `k_unMaxTrackedDeviceCount`, so it is never
/// `k_unTrackedDeviceIndexInvalid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceIndex(u8);

impl DeviceIndex {
    /// The index for a raw `TrackedDeviceIndex_t`, or `None` when it names no slot.
    pub fn new(raw: u32) -> Option<Self> {
        u8::try_from(raw)
            .ok()
            .map(Self)
            .filter(|index| index.slot() < MAX_DEVICES)
    }

    /// Every device slot, in order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..=u8::MAX).map(Self).take(MAX_DEVICES)
    }

    /// The raw `TrackedDeviceIndex_t`.
    pub fn get(self) -> u32 {
        u32::from(self.0)
    }

    /// The position in a per-device array.
    pub(crate) fn slot(self) -> usize {
        usize::from(self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::device::index::{DeviceIndex, MAX_DEVICES};

    #[test]
    fn covers_every_slot_and_nothing_else() {
        assert_eq!(DeviceIndex::all().count(), MAX_DEVICES);
        assert_eq!(DeviceIndex::new(63).map(DeviceIndex::get), Some(63));
        assert_eq!(DeviceIndex::new(64), None);
        assert_eq!(DeviceIndex::new(u32::MAX), None);
    }
}
