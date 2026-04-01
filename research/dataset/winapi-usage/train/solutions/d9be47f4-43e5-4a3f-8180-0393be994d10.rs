use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

fn call_get_module_file_name_w() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut buffer = [0u16; 260];
    let result = unsafe { GetModuleFileNameW(None, &mut buffer) };

    if result == 0 {
        return WIN32_ERROR(unsafe { GetLastError().0 });
    }

    WIN32_ERROR(0)
}
