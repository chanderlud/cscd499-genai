use windows::core::w;
use windows::Win32::System::Com::CLSIDFromProgID;

fn call_clsid_from_prog_id() -> windows::Win32::Foundation::WIN32_ERROR {
    let result = unsafe { CLSIDFromProgID(w!("Excel.Application")) };
    match result {
        Ok(_) => windows::Win32::Foundation::WIN32_ERROR(0),
        Err(e) => windows::Win32::Foundation::WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
