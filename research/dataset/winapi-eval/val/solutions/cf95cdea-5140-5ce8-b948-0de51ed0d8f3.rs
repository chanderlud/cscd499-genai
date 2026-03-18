use windows::core::{Error, Result, HRESULT};

pub fn same_os_error(win: &windows::core::Error, io: &std::io::Error) -> bool {
    // Get the HRESULT from the windows::core::Error
    let win_hresult = win.code();

    // Get the raw OS error from std::io::Error
    match io.raw_os_error() {
        Some(io_code) => {
            // Convert the raw OS error (i32) to HRESULT using the canonical conversion
            let io_hresult = HRESULT::from_win32(io_code as u32);

            // Compare the canonical HRESULTs
            win_hresult == io_hresult
        }
        None => false,
    }
}
