//! Self-update for official release builds: on start, compare with the newest GitHub release and,
//! when it is newer, swap the exe for it and relaunch.
//!
//! Only builds made with `TOUCH_DSU_RELEASE=1` (the release workflow) update themselves, so a
//! `cargo run` or `scripts\start.ps1` build is never replaced. `BRIDGE_NO_UPDATE=1` opts out at
//! run time. Any failure is logged and the current version carries on.

mod release;
mod replace;
mod version;

use std::env;
use std::io;
use std::process::{Command, ExitCode};

pub use version::Version;

const NO_UPDATE_VAR: &str = "BRIDGE_NO_UPDATE";
const OPT_OUT: &str = "1";

/// Whether this build updates itself: an official release, not opted out.
fn is_enabled() -> bool {
    let is_release_build = option_env!("TOUCH_DSU_RELEASE") == Some(OPT_OUT);
    let is_opted_out = env::var(NO_UPDATE_VAR).is_ok_and(|value| value == OPT_OUT);
    is_release_build && !is_opted_out
}

/// Deletes the files an earlier update left beside the exe.
pub fn remove_leftovers() {
    if let Ok(exe) = env::current_exe() {
        replace::remove_leftovers(&exe);
    }
}

/// Updates to the newest release when there is one. `Some(code)` means a newer exe was started
/// in this one's place and this process should exit with `code`; `None` means carry on.
pub fn run() -> Option<ExitCode> {
    if !is_enabled() {
        return None;
    }
    match update() {
        Ok(Some(code)) => Some(code),
        Ok(None) => None,
        Err(error) => {
            eprintln!("touch_dsu: update check failed ({error}); carrying on");
            None
        }
    }
}

fn update() -> io::Result<Option<ExitCode>> {
    let current = Version::current().ok_or_else(|| io::Error::other("bad build version"))?;
    let Some(latest) = release::latest()? else {
        println!("touch_dsu {current}: no release published yet");
        return Ok(None);
    };
    if latest <= current {
        println!("touch_dsu {current}: up to date");
        return Ok(None);
    }
    println!("touch_dsu {current}: updating to {latest}");
    let bytes = release::download(latest)?;
    if !replace::is_executable(&bytes) {
        return Err(io::Error::other(format!(
            "{} of {latest} is not an exe",
            release::ASSET_NAME
        )));
    }
    let exe = env::current_exe()?;
    replace::replace(&exe, &bytes)?;
    Command::new(&exe).args(env::args_os().skip(1)).spawn()?;
    Ok(Some(ExitCode::SUCCESS))
}
