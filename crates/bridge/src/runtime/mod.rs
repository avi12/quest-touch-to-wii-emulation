//! The 250 Hz loop: read the controllers, turn them into one DSU pad, send it to Dolphin.

mod clock;
mod error;
mod frame;
mod hands;
mod swing_hold;
mod throttle;

pub use error::BridgeError;

use std::convert::Infallible;
use std::thread;
use std::time::{Duration, Instant};

use dsu::{PadState, Server};
use openvr_sys::OpenVr;

use crate::config::Config;
use crate::input::Hotkeys;
use crate::motion::MotionTracker;
use crate::pointer::Pointer;
use crate::session::{SecondControllerWatcher, Telemetry};
use swing_hold::SwingHold;
use throttle::Throttle;

const RATE_HZ: f64 = 250.0;
const STEP_SECONDS: f64 = 1.0 / RATE_HZ;
const TIMER_RESOLUTION_MS: u32 = 1;

/// Everything one bridge instance owns between frames.
struct Bridge {
    config: Config,
    open_vr: OpenVr,
    server: Server,
    motion: MotionTracker,
    pointer: Pointer,
    hotkeys: Hotkeys,
    second_controller: SecondControllerWatcher,
    report: Throttle,
    is_left_enabled: bool,
    last_pad: Option<PadState>,
    telemetry: Telemetry,
    swing: SwingHold,
    /// The monotonic clock's zero, like `performance.now()`'s.
    started_at: Instant,
}

impl Bridge {
    fn new(config: Config) -> Result<Self, BridgeError> {
        let open_vr = OpenVr::new().map_err(BridgeError::OpenVr)?;
        let server = Server::bind(config.port).map_err(|source| BridgeError::Bind {
            port: config.port,
            source,
        })?;
        let telemetry_path = config.telemetry_path.as_deref();
        let telemetry =
            Telemetry::open(telemetry_path).map_err(|source| BridgeError::Telemetry {
                path: telemetry_path.map(ToOwned::to_owned).unwrap_or_default(),
                source,
            })?;
        win32_sys::set_timer_resolution(TIMER_RESOLUTION_MS);
        let role = if config.is_player2 {
            "(left controller = Wii Remote 2)"
        } else {
            "(right controller = Wii Remote 1)"
        };
        println!(
            "Bridge up on UDP {} {role} - waiting for controllers",
            config.port
        );
        let now = Instant::now();
        Ok(Self {
            hotkeys: Hotkeys::new(config.is_player2),
            config,
            open_vr,
            server,
            motion: MotionTracker::new(),
            pointer: Pointer::new(),
            second_controller: SecondControllerWatcher::new(now),
            report: Throttle::default(),
            is_left_enabled: true,
            last_pad: None,
            telemetry,
            swing: SwingHold::default(),
            started_at: now,
        })
    }

    /// Seconds on the monotonic clock since start-up.
    fn now_seconds(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64()
    }

    /// Answers whatever Dolphin asked since the last frame.
    fn poll_requests(&mut self) {
        if let Err(error) = self.server.poll() {
            eprintln!("DSU socket error: {error}");
        }
    }
}

/// Starts the bridge and runs it until the process ends; returns only on a start-up failure.
pub fn run_bridge(config: Config) -> Result<Infallible, BridgeError> {
    let mut bridge = Bridge::new(config)?;
    let step = Duration::from_secs_f64(STEP_SECONDS);
    let mut next_tick_at = bridge.now_seconds();
    loop {
        next_tick_at += STEP_SECONDS;
        bridge.poll_requests();
        if !bridge.tick() {
            thread::sleep(step);
            continue;
        }
        let wait_seconds = next_tick_at - bridge.now_seconds();
        if wait_seconds > 0.0 {
            thread::sleep(Duration::from_secs_f64(wait_seconds));
        }
    }
}
