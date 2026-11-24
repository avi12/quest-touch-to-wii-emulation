//! `WinHTTP`'s C API as `winhttp.h` declares it, and an owning handle.

use std::ffi::c_void;
use std::io;
use std::ptr::{self, NonNull};

// SAFETY: every signature matches winhttp.dll's export as `winhttp.h` declares it (`HINTERNET` is a
// pointer, `LPCWSTR` a NUL-terminated UTF-16 string, `DWORD` a u32, `INTERNET_PORT` a u16, `BOOL` an
// i32, `DWORD_PTR` a usize).
#[link(name = "winhttp")]
unsafe extern "system" {
    pub(crate) fn WinHttpOpen(
        agent: *const u16,
        access_type: u32,
        proxy: *const u16,
        proxy_bypass: *const u16,
        flags: u32,
    ) -> *mut c_void;
    pub(crate) fn WinHttpConnect(
        session: *mut c_void,
        server: *const u16,
        port: u16,
        reserved: u32,
    ) -> *mut c_void;
    pub(crate) fn WinHttpOpenRequest(
        connection: *mut c_void,
        verb: *const u16,
        object: *const u16,
        version: *const u16,
        referrer: *const u16,
        accept_types: *const *const u16,
        flags: u32,
    ) -> *mut c_void;
    pub(crate) fn WinHttpSetOption(
        handle: *mut c_void,
        option: u32,
        buffer: *const c_void,
        buffer_length: u32,
    ) -> i32;
    pub(crate) fn WinHttpSendRequest(
        request: *mut c_void,
        headers: *const u16,
        headers_length: u32,
        optional: *const c_void,
        optional_length: u32,
        total_length: u32,
        context: usize,
    ) -> i32;
    pub(crate) fn WinHttpReceiveResponse(request: *mut c_void, reserved: *mut c_void) -> i32;
    pub(crate) fn WinHttpQueryHeaders(
        request: *mut c_void,
        info_level: u32,
        name: *const u16,
        buffer: *mut c_void,
        buffer_length: *mut u32,
        index: *mut u32,
    ) -> i32;
    pub(crate) fn WinHttpQueryDataAvailable(request: *mut c_void, available: *mut u32) -> i32;
    pub(crate) fn WinHttpReadData(
        request: *mut c_void,
        buffer: *mut c_void,
        to_read: u32,
        read: *mut u32,
    ) -> i32;
    fn WinHttpCloseHandle(handle: *mut c_void) -> i32;
}

/// An open `WinHTTP` handle (session, connection or request), closed on drop.
pub(crate) struct Handle(NonNull<c_void>);

impl Handle {
    /// Wraps what a `WinHttp*` opener returned; a null handle is the call's error.
    pub(crate) fn from_raw(raw: *mut c_void) -> io::Result<Self> {
        NonNull::new(raw)
            .map(Self)
            .ok_or_else(io::Error::last_os_error)
    }

    pub(crate) const fn raw(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: the handle came from a WinHTTP opener and is closed exactly once, here.
        unsafe {
            WinHttpCloseHandle(self.0.as_ptr());
        }
    }
}

/// A `BOOL` result as `io::Result`, with the thread's last error on failure.
pub(crate) fn check(result: i32) -> io::Result<()> {
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// `text` as a NUL-terminated UTF-16 string for a `LPCWSTR` parameter.
pub(crate) fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

/// A null `LPCWSTR`: "use the default".
pub(crate) const fn no_text() -> *const u16 {
    ptr::null()
}
