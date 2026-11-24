//! Tracked-device poses: one snapshot of every device, decoded on demand.

use geometry::{Mat3, Vec3};

use crate::device::code::c_enum;
use crate::device::index::{DeviceIndex, MAX_DEVICES};
use crate::device::transform::{ComponentTransform, to_vec3};
use crate::ffi::table::{RawPose, SystemTable, len_u32};

/// `TrackingUniverseStanding`.
const TRACKING_STANDING: i32 = 1;

c_enum! {
    /// `ETrackingResult`: how well `SteamVR` is tracking a device.
    TrackingResult {
        /// Not tracked yet.
        Uninitialized = 1,
        /// Calibrating.
        CalibratingInProgress = 100,
        /// Calibrating, out of the tracked space.
        CalibratingOutOfRange = 101,
        /// Tracked.
        RunningOk = 200,
        /// Out of the tracked space; the position may be frozen.
        RunningOutOfRange = 201,
        /// Only the rotation is tracked.
        FallbackRotationOnly = 300,
    }
}

/// One tracked device: validity, rotation (3x3 rows) and position, plus `SteamVR`'s velocities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DevicePose {
    /// `SteamVR` has a valid pose for the device.
    pub is_valid: bool,
    /// How well the device is tracked; `None` for a value this binding does not know.
    pub tracking: Option<TrackingResult>,
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
            is_valid: raw.valid != 0,
            tracking: TrackingResult::from_code(raw.tracking_result),
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

    /// The pose of device `index` (invalid when the slot is empty).
    pub fn get(&self, index: DeviceIndex) -> DevicePose {
        // A `DeviceIndex` is always below `MAX_DEVICES`, so the default is never used.
        let raw = self.raw.get(index.slot()).copied().unwrap_or_default();
        DevicePose::from(&raw)
    }
}

#[cfg(test)]
mod tests {
    use crate::device::index::{DeviceIndex, MAX_DEVICES};
    use crate::device::pose::{DevicePose, Poses, TrackingResult};
    use crate::ffi::table::RawPose;

    const RUNNING_OK: i32 = 200;
    const RAW: RawPose = RawPose {
        device_to_absolute: [
            [1.0, 0.0, 0.0, 0.5],
            [0.0, 1.0, 0.0, 1.5],
            [0.0, 0.0, 1.0, -2.0],
        ],
        velocity: [0.25, 0.0, 0.0],
        angular_velocity: [0.0, 0.0, 3.0],
        tracking_result: RUNNING_OK,
        valid: 1,
        connected: 1,
    };

    #[test]
    fn decodes_a_raw_pose() {
        let pose = DevicePose::from(&RAW);
        assert!(pose.is_valid);
        assert_eq!(pose.tracking, Some(TrackingResult::RunningOk));
        assert_eq!(
            pose.rot,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        );
        assert_eq!(pose.pos, [0.5, 1.5, -2.0]);
        assert_eq!(pose.velocity, [0.25, 0.0, 0.0]);
        assert_eq!(pose.angular_velocity, [0.0, 0.0, 3.0]);
    }

    #[test]
    fn every_index_reads_a_slot() {
        let poses = Poses {
            raw: [RAW; MAX_DEVICES],
        };
        assert!(DeviceIndex::all().all(|index| poses.get(index).is_valid));
    }
}
