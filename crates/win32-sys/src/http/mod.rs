//! A minimal blocking HTTPS GET through the `WinHTTP` that ships with Windows, so the workspace
//! needs no TLS crate.

mod ffi;
mod response;
mod url;

use std::ffi::c_void;
use std::io;
use std::ptr;

pub use response::Response;
pub use url::HttpsUrl;

use crate::http::ffi::{
    Handle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpReceiveResponse,
    WinHttpSendRequest, WinHttpSetOption, check, no_text, wide,
};

/// HTTP 200.
pub const STATUS_OK: u16 = 200;
/// `WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY`: the system's proxy settings.
const ACCESS_AUTOMATIC_PROXY: u32 = 4;
/// `WINHTTP_FLAG_SECURE`: TLS.
const FLAG_SECURE: u32 = 0x0080_0000;
/// `WINHTTP_OPTION_DISABLE_FEATURE`.
const OPTION_DISABLE_FEATURE: u32 = 63;
/// `WINHTTP_DISABLE_REDIRECTS`.
const DISABLE_REDIRECTS: u32 = 0x0000_0002;

/// Whether a 3xx answer is followed or returned as is (its `Location` then says where).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redirects {
    /// Follow redirects to the final answer.
    Follow,
    /// Return the redirect itself.
    Manual,
}

/// GETs `url` (HTTPS only) and reads the whole body.
pub fn get(url: &str, user_agent: &str, redirects: Redirects) -> io::Result<Response> {
    let target = HttpsUrl::parse(url).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not an https URL: {url}"),
        )
    })?;
    let agent = wide(user_agent);
    // SAFETY: the agent string is NUL-terminated UTF-16 that outlives the call; null = default.
    let session = Handle::from_raw(unsafe {
        WinHttpOpen(
            agent.as_ptr(),
            ACCESS_AUTOMATIC_PROXY,
            no_text(),
            no_text(),
            0,
        )
    })?;
    let host = wide(&target.host);
    // SAFETY: `session` is open; the host string outlives the call.
    let connection =
        Handle::from_raw(unsafe { WinHttpConnect(session.raw(), host.as_ptr(), target.port, 0) })?;
    let verb = wide("GET");
    let path = wide(&target.path);
    // SAFETY: `connection` is open; the strings outlive the call; null accept types = any.
    let request = Handle::from_raw(unsafe {
        WinHttpOpenRequest(
            connection.raw(),
            verb.as_ptr(),
            path.as_ptr(),
            no_text(),
            no_text(),
            ptr::null(),
            FLAG_SECURE,
        )
    })?;
    if redirects == Redirects::Manual {
        disable_redirects(&request)?;
    }
    // SAFETY: `request` is open; no extra headers and no body.
    check(unsafe { WinHttpSendRequest(request.raw(), no_text(), 0, ptr::null(), 0, 0, 0) })?;
    // SAFETY: `request` was sent; the reserved argument must be null.
    check(unsafe { WinHttpReceiveResponse(request.raw(), ptr::null_mut()) })?;
    Response::read(&request)
}

fn disable_redirects(request: &Handle) -> io::Result<()> {
    let flags = DISABLE_REDIRECTS;
    let length = u32::try_from(size_of::<u32>()).unwrap_or(u32::MAX);
    // SAFETY: the option takes a DWORD buffer of the length passed, alive for the call.
    check(unsafe {
        WinHttpSetOption(
            request.raw(),
            OPTION_DISABLE_FEATURE,
            (&raw const flags).cast::<c_void>(),
            length,
        )
    })
}
