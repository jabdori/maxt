pub mod api;
#[allow(clippy::result_large_err)] // Generated code serializes public error information by value.
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

use std::ffi::{CString, c_char};
use std::sync::OnceLock;

mod adapter;
mod convert;
mod stream;

static NATIVE_LIBRARY_PATH: OnceLock<Option<CString>> = OnceLock::new();

/// Symbol that lets Native Assets load the library before a standalone Dart process starts.
#[unsafe(no_mangle)]
pub extern "C" fn maxt_dart_bridge_force_load() {}

/// Returns the path of the dynamic library containing this symbol.
///
/// The returned pointer remains valid until process exit and must not be freed.
/// Returns null when the path cannot be determined.
#[unsafe(no_mangle)]
pub extern "C" fn maxt_dart_bridge_library_path() -> *const c_char {
    NATIVE_LIBRARY_PATH
        .get_or_init(discover_native_library_path)
        .as_ref()
        .map_or(std::ptr::null(), |path| path.as_ptr())
}

#[cfg(unix)]
fn discover_native_library_path() -> Option<CString> {
    use std::ffi::CStr;
    use std::mem::MaybeUninit;

    let mut info = MaybeUninit::<libc::Dl_info>::uninit();
    let address = maxt_dart_bridge_library_path as *const () as *const libc::c_void;

    // SAFETY: successful dladdr initializes `info`, and the function address belongs to this module.
    if unsafe { libc::dladdr(address, info.as_mut_ptr()) } == 0 {
        return None;
    }

    // SAFETY: a successful dladdr call initializes `info` and a null-terminated path.
    let path = unsafe { info.assume_init().dli_fname };
    if path.is_null() {
        return None;
    }

    // Native Assets paths are UTF-8, but reject malformed platform paths before the FFI boundary.
    CString::new(unsafe { CStr::from_ptr(path) }.to_string_lossy().as_bytes()).ok()
}

#[cfg(windows)]
fn discover_native_library_path() -> Option<CString> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };

    let mut module = std::ptr::null_mut();
    let address = maxt_dart_bridge_library_path as *const () as *const u16;
    let flags =
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;

    // SAFETY: the FROM_ADDRESS flag interprets the second argument as an address in this module.
    if unsafe { GetModuleHandleExW(flags, address, &mut module) } == 0 {
        return None;
    }

    let mut capacity = 260;
    while capacity <= 32_768 {
        let mut path = vec![0_u16; capacity];
        // SAFETY: the buffer can hold `capacity` UTF-16 code units.
        let length = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), capacity as u32) };
        if length == 0 {
            return None;
        }
        if length < capacity as u32 {
            return CString::new(String::from_utf16_lossy(&path[..length as usize])).ok();
        }
        capacity *= 2;
    }

    None
}

#[cfg(not(any(unix, windows)))]
fn discover_native_library_path() -> Option<CString> {
    None
}

#[cfg(test)]
mod tests {
    use super::maxt_dart_bridge_library_path;
    use std::ffi::CStr;

    #[test]
    fn native_library_path_is_owned_for_the_process_lifetime() {
        let first = maxt_dart_bridge_library_path();
        let second = maxt_dart_bridge_library_path();

        assert!(!first.is_null());
        assert_eq!(first, second);
        // SAFETY: the public function contract keeps the returned pointer valid until process exit.
        assert!(!unsafe { CStr::from_ptr(first) }.to_bytes().is_empty());
    }
}
