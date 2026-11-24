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
use win32_sys::TimerResolution;

use crate::config::Config;
use crate::input::Hotkeys;
use crate::motion::MotionTracker;
use crate::pointer::Pointer;
use crate::runtime::swing_hold::SwingHold;
use crate::runtime::throttle::Throttle;
use crate::session::{SecondControllerWatcher, Telemetry};

/// One frame of the 250 Hz loop.
const STEP: Duration = Duration::from_millis(4);
const TIMER_RESOLUTION: Duration = Duration::from_millis(1);

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
    /// Finer Windows timer ticks for the bridge's whole life; `None` if Windows refused them.
    _timer_resolution: Option<TimerResolution>,
}

impl Bridge {
    fn new(config: Config) -> Result<Self, BridgeError> {
        let open_vr = OpenVr::new().map_err(BridgeError::OpenVr)?;
        let server = Server::bind(config.port.get()).map_err(|source| BridgeError::Bind {
            port: config.port,
            source,
        })?;
        let telemetry_path = config.telemetry_path.as_deref();
        let telemetry =
            Telemetry::open(telemetry_path).map_err(|source| BridgeError::Telemetry {
                path: telemetry_path.map(ToOwned::to_owned).unwrap_or_default(),
                source,
            })?;
        let timer_resolution = win32_sys::set_timer_resolution(TIMER_RESOLUTION);
        let role = if config.is_player2() {
            "(left controller = Wii Remote 2)"
        } else {
            "(right controller = Wii Remote 1)"
        };
        println!(
            "Bridge up on UDP {} {role} - waiting for controllers",
            config.port.get()
        );
        let now = Instant::now();
        Ok(Self {
            hotkeys: Hotkeys::new(config.hand),
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
            _timer_resolution: timer_resolution,
        })
    }

    /// The monotonic clock: time since start-up.
    fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
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
    let mut next_tick_at = bridge.elapsed();
    loop {
        next_tick_at += STEP;
        bridge.poll_requests();
        if !bridge.tick() {
            thread::sleep(STEP);
            continue;
        }
        if let Some(wait) = next_tick_at.checked_sub(bridge.elapsed()) {
            thread::sleep(wait);
        }
    }
}
