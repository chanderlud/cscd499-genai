#![deny(warnings)]

use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

#[allow(dead_code)]
fn call_get_module_handle_w() -> windows::core::HRESULT {
    unsafe {
        match GetModuleHandleW(PCWSTR::null()) {
            Ok(_) => windows::core::HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
