//! One frame of the loop: controllers in, pad out.

use std::time::{Duration, Instant};

use dsu::PadState;
use openvr_sys::{Button, DevicePose, Hand};

use super::clock::epoch_ms;
use super::hands::Output;
use super::{Bridge, STEP_SECONDS};
use crate::config::hand_name;
use crate::input::{Controllers, PadInput, build_pad};
use crate::math::round_hundredths;
use crate::motion::{PoseSample, Sensors, to_sensors};
use crate::pointer::{PointerInput, PointerPosition};
use crate::session::TelemetrySample;

/// How long Dolphin's IMU "Recenter" (the PS button) is held after a recenter.
const RECENTER_HOLD: Duration = Duration::from_millis(300);
/// The pad repeated while the controller is lost carries no rotation.
const NO_GYRO: [f64; 3] = [0.0; 3];

impl Bridge {
    /// One pad update; false when the controller is not tracked.
    pub(super) fn tick(&mut self) -> bool {
        let poses = self.open_vr.poses();
        let pose_of = |index: Option<u32>| {
            index
                .and_then(|index| poses.get(index))
                .filter(|pose| pose.valid)
        };
        let right_index = self.open_vr.controller_index(self.config.hand);
        let left_index = if self.config.is_player2 {
            None
        } else {
            self.open_vr.controller_index(Hand::Left)
        };
        let (Some(index), Some(pose)) = (right_index, pose_of(right_index)) else {
            self.hold_last_pad();
            return false;
        };
        let is_left_tracked = pose_of(left_index).is_some();
        if !self.config.is_player2 {
            self.second_controller
                .update(is_left_tracked, Instant::now());
        }
        self.apply_hotkeys();
        let controllers = self.read_controllers(index, left_index);
        self.send_frame(&pose, controllers, is_left_tracked);
        true
    }

    fn send_frame(&mut self, pose: &DevicePose, controllers: Controllers, is_left_tracked: bool) {
        let sensors = self.sense(pose);
        if self.swing.observe(sensors.accel, Instant::now()) {
            self.server.recenter_until = Instant::now() + RECENTER_HOLD;
        }
        let Controllers { right, left } = controllers;
        let pointer = self.pointer.update(&PointerInput {
            rotation: pose.rot,
            is_click_pressed: left.is_down(Button::Click) || right.is_down(Button::Click),
            delta_seconds: STEP_SECONDS,
            now_seconds: self.now_seconds(),
        });
        if pointer.did_recenter {
            println!("RECENTER");
            self.server.recenter_until = Instant::now() + RECENTER_HOLD;
        }
        let held = self.swing.hold(&pointer, Instant::now());
        let output = self.apply_overrides(Output {
            pointer: held,
            sensors,
        });
        let now_epoch_ms = epoch_ms();
        let pad = build_pad(&PadInput {
            controllers,
            is_left_attached: !self.config.is_player2 && self.is_left_enabled && is_left_tracked,
            pointer: output.pointer,
            sensors: output.sensors,
            now_epoch_ms,
        });
        self.last_pad = Some(PadState {
            gyro: NO_GYRO,
            ..pad
        });
        self.send(&pad);
        let sample = TelemetrySample {
            direction: pointer.direction,
            pointer: output.pointer,
            sensors: output.sensors,
            did_recenter: pointer.did_recenter,
            rotation: pose.rot,
            facing: pointer.facing,
            is_marked: right.is_down(Button::Secondary),
        };
        self.record(&sample, now_epoch_ms);
        if self.report.is_due(Instant::now()) {
            self.log_status(is_left_tracked, output.pointer);
        }
    }

    /// The Wii Remote's accelerometer and gyro for this pose.
    fn sense(&mut self, pose: &DevicePose) -> Sensors {
        let motion = self.motion.update(&PoseSample {
            position: pose.pos,
            rotation: pose.rot,
            velocity: pose.velocity,
            angular_velocity: pose.angular_velocity,
            timestamp_seconds: self.now_seconds(),
        });
        to_sensors(pose.rot, self.pointer.basis(), motion)
    }

    fn record(&mut self, sample: &TelemetrySample, now_epoch_ms: u64) {
        if let Err(error) = self.telemetry.record(sample, now_epoch_ms) {
            eprintln!("telemetry write failed: {error}");
        }
    }

    fn send(&mut self, pad: &PadState) {
        if let Err(error) = self.server.send_pad(pad) {
            eprintln!("DSU send failed: {error}");
        }
    }

    fn hold_last_pad(&mut self) {
        if self.report.is_due(Instant::now()) {
            println!(
                "{} controller not tracked - holding last pose (keep it in front of the headset cameras)",
                hand_name(self.config.hand)
            );
        }
        if let Some(pad) = self.last_pad {
            self.send(&pad);
        }
    }

    fn log_status(&self, is_left_tracked: bool, pointer: PointerPosition) {
        let state = if self.is_left_enabled {
            "ON (used)"
        } else {
            "OFF (ignored)"
        };
        println!(
            "left controller: {state} | seen by SteamVR: {is_left_tracked} | dolphin connected: {} | ptr {} {}",
            self.server.client_count() > 0,
            round_hundredths(pointer.x),
            round_hundredths(pointer.y)
        );
    }
}
