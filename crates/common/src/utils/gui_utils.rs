use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use winapi::um::winuser::*;
/// 提示框
pub fn show_message_box(title: &str, message: &str) {
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            to_wstring(message).as_ptr(),
            to_wstring(title).as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

/// 错误提示框
pub fn show_error_box(title: &str, message: &str) {
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            to_wstring(message).as_ptr(),
            to_wstring(title).as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

/// 字符串转宽字符
pub fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}