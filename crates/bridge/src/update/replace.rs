//! Swapping the running exe for a downloaded one. Windows lets a running exe be renamed but not
//! overwritten, so the current file steps aside as `.old` and is deleted on the next start.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Every Windows executable starts with these two bytes.
const EXE_MAGIC: &[u8] = b"MZ";
const NEW_SUFFIX: &str = "new";
const OLD_SUFFIX: &str = "old";

/// Whether `bytes` look like a Windows executable rather than, say, an error page.
pub(crate) fn is_executable(bytes: &[u8]) -> bool {
    bytes.starts_with(EXE_MAGIC)
}

fn beside(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.as_os_str().to_owned();
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

/// Puts `bytes` in place of `exe`; on failure the original stays where it was.
pub(crate) fn replace(exe: &Path, bytes: &[u8]) -> io::Result<()> {
    let staged = beside(exe, NEW_SUFFIX);
    let retired = beside(exe, OLD_SUFFIX);
    fs::write(&staged, bytes)?;
    // A previous update's leftover would block the rename below.
    let _ignored = fs::remove_file(&retired);
    fs::rename(exe, &retired)?;
    if let Err(error) = fs::rename(&staged, exe) {
        let _restored = fs::rename(&retired, exe);
        return Err(error);
    }
    Ok(())
}

/// Deletes what an earlier update left behind; harmless when there is nothing.
pub(crate) fn remove_leftovers(exe: &Path) {
    for suffix in [OLD_SUFFIX, NEW_SUFFIX] {
        let _ignored = fs::remove_file(beside(exe, suffix));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::update::replace::{is_executable, remove_leftovers, replace};

    #[test]
    fn recognises_executables() {
        assert!(is_executable(b"MZ\x90\x00"));
        assert!(!is_executable(b"<!DOCTYPE html>"));
        assert!(!is_executable(b""));
    }

    #[test]
    fn swaps_the_file_and_cleans_up() {
        let folder = std::env::temp_dir().join(format!("touch_dsu-replace-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let exe = folder.join("touch_dsu.exe");
        fs::write(&exe, b"MZ old").unwrap();
        replace(&exe, b"MZ new").unwrap();
        assert_eq!(fs::read(&exe).unwrap(), b"MZ new");
        remove_leftovers(&exe);
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
        fs::remove_dir_all(&folder).unwrap();
    }
}
