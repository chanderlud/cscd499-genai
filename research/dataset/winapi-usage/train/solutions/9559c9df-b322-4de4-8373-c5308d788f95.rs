use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

fn call_get_module_file_name_w() -> HRESULT {
    let mut buffer = [0u16; 260];

    let result = unsafe { GetModuleFileNameW(None, &mut buffer) };

    if result == 0 {
        Error::from_thread().code()
    } else {
        S_OK
    }
}
