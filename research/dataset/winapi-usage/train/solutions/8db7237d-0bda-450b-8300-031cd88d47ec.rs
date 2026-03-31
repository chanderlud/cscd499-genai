use windows::core::{Error, Result};
use windows::Win32::System::LibraryLoader::BeginUpdateResourceA;

fn call_begin_update_resource_a() -> windows::core::HRESULT {
    unsafe {
        match BeginUpdateResourceA(windows::core::s!("test.exe"), false) {
            Ok(_) => windows::core::HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
