use std::ffi::CStr;
use std::fmt::Formatter;
use std::os::raw::{c_int, c_char, c_void};

pub unsafe extern "C" fn stream_writer(data: *const c_char, length: c_int, stream: *mut c_void) -> c_int {
    let f: &mut Formatter = std::mem::transmute(stream);
    let string = CStr::from_bytes_with_nul_unchecked(
        std::slice::from_raw_parts(data as *const u8, length as usize + 1)
    );

    let result = f.write_str(string.to_string_lossy().as_ref());
    match result {
        Ok(_) => 0,
        Err(_) => -1
    }
}