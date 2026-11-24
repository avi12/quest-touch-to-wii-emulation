//! Finding and loading `openvr_api.dll`, initialising `OpenVR` and fetching function tables.

use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::{self, NonNull};

use libloading::Library;

use crate::error::OpenVrError;

const STEAM_DIRS: [&str; 2] = ["C:/Program Files (x86)/Steam", "C:/Program Files/Steam"];
const DLL_SUBPATH: &str = "steamapps/common/SteamVR/bin/win64/openvr_api.dll";
/// `VRApplication_Background`: no display, no scene.
const APPLICATION_BACKGROUND: i32 = 3;

type InitFn = unsafe extern "C" fn(*mut i32, i32, *const c_char) -> isize;
type GetInterfaceFn = unsafe extern "C" fn(*const c_char, *mut i32) -> *mut c_void;
type ErrorSymbolFn = unsafe extern "C" fn(i32) -> *const c_char;
type ShutdownFn = unsafe extern "C" fn();

/// The loaded DLL with `OpenVR` initialised; shuts `OpenVR` down when dropped.
pub(crate) struct Runtime {
    get_interface: GetInterfaceFn,
    shutdown: ShutdownFn,
    // Fields drop after `Drop::drop`, so the DLL stays loaded until `shutdown` has run.
    _library: Library,
}

fn find_dll() -> Option<PathBuf> {
    STEAM_DIRS
        .iter()
        .map(|dir| Path::new(dir).join(DLL_SUBPATH))
        .find(|path| path.exists())
}

/// Copies an entry point out of the library.
///
/// # Safety
/// `F` must be the entry point's exact function-pointer type, and the copy must not be called
/// after `library` is unloaded.
unsafe fn entry<F: Copy>(library: &Library, name: &CStr) -> Result<F, OpenVrError> {
    // SAFETY: the caller guarantees `F` matches the symbol.
    let symbol = unsafe { library.get::<F>(name.to_bytes_with_nul())? };
    Ok(*symbol)
}

fn error_symbol(describe: ErrorSymbolFn, code: i32) -> String {
    // SAFETY: VR_GetVRInitErrorAsSymbol takes any code and returns a static string or null.
    let text = unsafe { describe(code) };
    if text.is_null() {
        return String::new();
    }

    // SAFETY: a non-null result is a NUL-terminated static string owned by the DLL.
    unsafe { CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

impl Runtime {
    /// Loads the DLL and initialises `OpenVR` as a background application.
    pub(crate) fn init() -> Result<Self, OpenVrError> {
        let path = find_dll().ok_or(OpenVrError::DllNotFound)?;
        // SAFETY: loading openvr_api.dll runs no initialisation with preconditions of ours.
        let library = unsafe { Library::new(path)? };
        // SAFETY: the types match openvr_capi.h, and `Runtime` keeps `library` loaded for as
        // long as it holds the copies.
        let (init, get_interface, describe, shutdown) = unsafe {
            (
                entry::<InitFn>(&library, c"VR_InitInternal2")?,
                entry::<GetInterfaceFn>(&library, c"VR_GetGenericInterface")?,
                entry::<ErrorSymbolFn>(&library, c"VR_GetVRInitErrorAsSymbol")?,
                entry::<ShutdownFn>(&library, c"VR_ShutdownInternal")?,
            )
        };
        let mut code = 0;
        // SAFETY: `code` is a valid out-pointer; null startup info is allowed.
        unsafe { init(&raw mut code, APPLICATION_BACKGROUND, ptr::null()) };
        if code != 0 {
            let symbol = error_symbol(describe, code);
            return Err(OpenVrError::Init { symbol, code });
        }

        Ok(Self {
            get_interface,
            shutdown,
            _library: library,
        })
    }

    /// The function table of one interface, such as `FnTable:IVRSystem_023`.
    pub(crate) fn table(&self, name: &CStr) -> Result<NonNull<c_void>, OpenVrError> {
        let mut code = 0;
        // SAFETY: `name` is NUL-terminated and `code` is a valid out-pointer.
        let table = unsafe { (self.get_interface)(name.as_ptr(), &raw mut code) };
        let found = if code == 0 { NonNull::new(table) } else { None };
        found.ok_or_else(|| interface_error(name, code))
    }
}

/// The error for an interface that is unavailable or has a null slot.
pub(crate) fn interface_error(name: &CStr, code: i32) -> OpenVrError {
    OpenVrError::Interface {
        name: name.to_string_lossy().into_owned(),
        code,
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // SAFETY: OpenVR was initialised by `init`, and the library is still loaded.
        unsafe { (self.shutdown)() };
    }
}
