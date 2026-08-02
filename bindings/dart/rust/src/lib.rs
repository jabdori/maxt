pub mod api;
#[allow(clippy::result_large_err)] // 생성 코드는 공개 오류 정보를 값으로 직렬화합니다.
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

use std::ffi::{CString, c_char};
use std::sync::OnceLock;

mod adapter;
mod convert;
mod stream;

static NATIVE_LIBRARY_PATH: OnceLock<Option<CString>> = OnceLock::new();

/// Native Assets가 standalone Dart 프로세스에 라이브러리를 먼저 적재하도록 하는 심벌입니다.
#[unsafe(no_mangle)]
pub extern "C" fn maxt_dart_bridge_force_load() {}

/// 현재 심벌을 포함하는 동적 라이브러리의 경로를 반환합니다.
///
/// 반환 포인터는 프로세스가 종료될 때까지 유효하며 해제하면 안 됩니다. 경로를 확인할
/// 수 없으면 null을 반환합니다.
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

    // SAFETY: `info`는 dladdr가 성공했을 때 초기화되며, 함수 주소는 현재 모듈에 속합니다.
    if unsafe { libc::dladdr(address, info.as_mut_ptr()) } == 0 {
        return None;
    }

    // SAFETY: 성공한 dladdr 호출은 `info`와 null 종료 경로를 초기화합니다.
    let path = unsafe { info.assume_init().dli_fname };
    if path.is_null() {
        return None;
    }

    // Native Assets 경로는 UTF-8이지만, 손상된 플랫폼 경로가 FFI 경계를 넘지 않게 합니다.
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

    // SAFETY: FROM_ADDRESS 플래그에서는 두 번째 인자를 모듈 내부 주소로 해석합니다.
    if unsafe { GetModuleHandleExW(flags, address, &mut module) } == 0 {
        return None;
    }

    let mut capacity = 260;
    while capacity <= 32_768 {
        let mut path = vec![0_u16; capacity];
        // SAFETY: 버퍼는 `capacity`개의 UTF-16 코드 단위를 쓸 수 있습니다.
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
        // SAFETY: 공개 함수 계약상 반환 포인터는 프로세스가 종료될 때까지 유효합니다.
        assert!(!unsafe { CStr::from_ptr(first) }.to_bytes().is_empty());
    }
}
