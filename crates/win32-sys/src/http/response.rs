//! Reading a `WinHTTP` response: status, `Location` and body.

use std::ffi::c_void;
use std::io;
use std::ptr;

use crate::http::ffi::{
    Handle, WinHttpQueryDataAvailable, WinHttpQueryHeaders, WinHttpReadData, check, no_text,
};

/// `WINHTTP_QUERY_STATUS_CODE`.
const QUERY_STATUS_CODE: u32 = 19;
/// `WINHTTP_QUERY_LOCATION`.
const QUERY_LOCATION: u32 = 33;
/// `WINHTTP_QUERY_FLAG_NUMBER`: answer as a DWORD instead of text.
const QUERY_AS_NUMBER: u32 = 0x2000_0000;
const BYTES_PER_WIDE_CHAR: usize = 2;

/// A finished HTTP answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    /// HTTP status, such as 200 or 302.
    pub status: u16,
    /// The `Location` header of a redirect.
    pub location: Option<String>,
    /// The whole body.
    pub body: Vec<u8>,
}

impl Response {
    pub(crate) fn read(request: &Handle) -> io::Result<Self> {
        Ok(Self {
            status: status(request)?,
            location: location(request),
            body: body(request)?,
        })
    }
}

fn status(request: &Handle) -> io::Result<u16> {
    let mut code = 0_u32;
    let mut length = u32::try_from(size_of::<u32>()).unwrap_or(u32::MAX);
    // SAFETY: a numeric query writes one DWORD into `code`, whose size is passed.
    check(unsafe {
        WinHttpQueryHeaders(
            request.raw(),
            QUERY_STATUS_CODE | QUERY_AS_NUMBER,
            no_text(),
            (&raw mut code).cast::<c_void>(),
            &raw mut length,
            ptr::null_mut(),
        )
    })?;
    u16::try_from(code).map_err(|_overflow| io::Error::other("HTTP status out of range"))
}

/// The `Location` header, or `None` when the response has none.
fn location(request: &Handle) -> Option<String> {
    let mut length = 0_u32;
    // SAFETY: a null buffer only asks for the needed length in bytes; that call fails by design.
    unsafe {
        WinHttpQueryHeaders(
            request.raw(),
            QUERY_LOCATION,
            no_text(),
            ptr::null_mut(),
            &raw mut length,
            ptr::null_mut(),
        );
    }
    let chars = usize::try_from(length).ok()? / BYTES_PER_WIDE_CHAR;
    if chars == 0 {
        return None;
    }
    let mut buffer = vec![0_u16; chars];
    // SAFETY: `buffer` holds `length` bytes, as the first call asked for.
    check(unsafe {
        WinHttpQueryHeaders(
            request.raw(),
            QUERY_LOCATION,
            no_text(),
            buffer.as_mut_ptr().cast::<c_void>(),
            &raw mut length,
            ptr::null_mut(),
        )
    })
    .ok()?;
    let written = usize::try_from(length).ok()? / BYTES_PER_WIDE_CHAR;
    String::from_utf16(buffer.get(..written)?).ok()
}

fn body(request: &Handle) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let mut available = 0_u32;
        // SAFETY: writes one DWORD into `available`.
        check(unsafe { WinHttpQueryDataAvailable(request.raw(), &raw mut available) })?;
        if available == 0 {
            return Ok(body);
        }
        let start = body.len();
        let chunk = usize::try_from(available).map_err(io::Error::other)?;
        body.resize(start + chunk, 0);
        let mut read = 0_u32;
        let tail = body.get_mut(start..).unwrap_or_default();
        // SAFETY: `tail` has room for `available` bytes, the most WinHTTP writes.
        check(unsafe {
            WinHttpReadData(
                request.raw(),
                tail.as_mut_ptr().cast::<c_void>(),
                available,
                &raw mut read,
            )
        })?;
        body.truncate(start + usize::try_from(read).map_err(io::Error::other)?);
    }
}
