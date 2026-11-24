//! Tracked-device poses: one snapshot of every device, decoded on demand.

use crate::device::transform::{ComponentTransform, Mat3, Vec3, to_vec3};
use crate::ffi::table::{RawPose, SystemTable, len_u32};

/// How many tracked devices `OpenVR` reports (`k_unMaxTrackedDeviceCount`).
pub const MAX_DEVICES: usize = 64;
/// `TrackingUniverseStanding`.
const TRACKING_STANDING: i32 = 1;

/// One tracked device: validity, rotation (3x3 rows) and position, plus `SteamVR`'s velocities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DevicePose {
    /// `SteamVR` has a valid pose for the device.
    pub valid: bool,
    /// The raw `ETrackingResult`.
    pub tracking_result: i32,
    /// Device-to-world rotation, row-major.
    pub rot: Mat3,
    /// Position in the standing universe, metres.
    pub pos: Vec3,
    /// m/s, world frame, as `SteamVR` reports it.
    pub velocity: Vec3,
    /// rad/s, world frame, as `SteamVR` reports it.
    pub angular_velocity: Vec3,
}

impl From<&RawPose> for DevicePose {
    fn from(raw: &RawPose) -> Self {
        let ComponentTransform { rot, origin } =
            ComponentTransform::from_matrix(&raw.device_to_absolute);
        Self {
            valid: raw.valid != 0,
            tracking_result: raw.tracking_result,
            rot,
            pos: origin,
            velocity: to_vec3(raw.velocity),
            angular_velocity: to_vec3(raw.angular_velocity),
        }
    }
}

/// Snapshot of every device's pose; each one is decoded only when asked for.
#[derive(Clone)]
pub struct Poses {
    raw: [RawPose; MAX_DEVICES],
}

impl Poses {
    /// Reads every device's current pose (no prediction).
    pub(crate) fn fetch(system: &SystemTable) -> Self {
        let mut raw = [RawPose::default(); MAX_DEVICES];
        // SAFETY: `raw` holds exactly the number of poses passed as the count.
        unsafe {
            (system.get_poses)(TRACKING_STANDING, 0.0, raw.as_mut_ptr(), len_u32(raw.len()));
        }
        Self { raw }
    }

    /// The pose of device `index`, or `None` past [`MAX_DEVICES`].
    pub fn get(&self, index: u32) -> Option<DevicePose> {
        let slot = usize::try_from(index).ok()?;
        self.raw.get(slot).map(DevicePose::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_raw_pose() {
        let raw = RawPose {
            device_to_absolute: [
                [1.0, 0.0, 0.0, 0.5],
                [0.0, 1.0, 0.0, 1.5],
                [0.0, 0.0, 1.0, -2.0],
            ],
            velocity: [0.25, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 3.0],
            tracking_result: 200,
            valid: 1,
            connected: 1,
        };
        let pose = DevicePose::from(&raw);
        assert!(pose.valid);
        assert_eq!(pose.tracking_result, 200);
        assert_eq!(
            pose.rot,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        );
        assert_eq!(pose.pos, [0.5, 1.5, -2.0]);
        assert_eq!(pose.velocity, [0.25, 0.0, 0.0]);
        assert_eq!(pose.angular_velocity, [0.0, 0.0, 3.0]);
    }

    #[test]
    fn out_of_range_index_is_none() {
        let poses = Poses {
            raw: [RawPose::default(); MAX_DEVICES],
        };
        assert!(poses.get(63).is_some());
        assert!(poses.get(64).is_none());
    }
}
