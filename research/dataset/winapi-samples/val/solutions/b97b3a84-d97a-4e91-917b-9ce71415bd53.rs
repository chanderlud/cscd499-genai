use windows::core::{Error, HRESULT};

pub fn same_os_error(win: &Error, io: &std::io::Error) -> bool {
    let hr_win = win.code();
    let raw_io = match io.raw_os_error() {
        Some(code) => code,
        None => return false,
    };
    // Convert Win32 error code to canonical HRESULT manually
    let hr_io = (raw_io as u32 & 0xFFFF) | (7 << 16) | 0x80000000;
    (hr_win.0 as u32) == hr_io
}
