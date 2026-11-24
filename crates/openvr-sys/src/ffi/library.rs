//! A loaded DLL and its entry points, through kernel32's `LoadLibraryW`, `GetProcAddress` and
//! `FreeLibrary`.

use std::ffi::{CStr, c_char, c_void};
use std::io;
use std::iter;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::NonNull;

// SAFETY: the signatures match kernel32's exports as `libloaderapi.h` declares them (`HMODULE`
// and `FARPROC` are pointers, `BOOL` is an `i32`).
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(file_name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

/// A DLL kept loaded until dropped.
#[derive(Debug)]
pub(crate) struct Library(NonNull<c_void>);

impl Library {
    /// Loads the DLL at `path`; the error is the OS's reason.
    pub(crate) fn load(path: &Path) -> io::Result<Self> {
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 path that outlives the call.
        let module = unsafe { LoadLibraryW(wide.as_ptr()) };
        NonNull::new(module)
            .map(Self)
            .ok_or_else(io::Error::last_os_error)
    }

    /// Copies an exported function pointer out of the DLL, or `None` when it has no such export.
    ///
    /// # Safety
    /// `F` must be the export's exact `extern` function-pointer type, and the copy must not be
    /// called after `self` is dropped.
    pub(crate) unsafe fn entry<F: Copy>(&self, name: &CStr) -> Option<F> {
        const { assert!(size_of::<F>() == size_of::<*mut c_void>()) };
        // SAFETY: the module is loaded while `self` lives, and `name` is NUL-terminated.
        let address = NonNull::new(unsafe { GetProcAddress(self.0.as_ptr(), name.as_ptr()) })?;
        // SAFETY: the caller guarantees `F` is the export's function-pointer type, which has the
        // size of an address (checked above).
        Some(unsafe { mem::transmute_copy::<NonNull<c_void>, F>(&address) })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: the handle came from `LoadLibraryW` and is freed exactly once, here.
        unsafe { FreeLibrary(self.0.as_ptr()) };
    }
}
