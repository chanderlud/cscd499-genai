use windows::core::{Error, Result};
use windows::Win32::Foundation::FALSE;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

fn main() -> Result<()> {
    let mut msg = MSG::default();

    // Main message loop
    loop {
        let ret = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if ret == FALSE {
            break;
        } else if ret.0 < 0 {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                unsafe { windows::Win32::Foundation::GetLastError() }.0,
            )));
        }

        // Translate and dispatch message
        unsafe {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        };
    }

    Ok(())
}
