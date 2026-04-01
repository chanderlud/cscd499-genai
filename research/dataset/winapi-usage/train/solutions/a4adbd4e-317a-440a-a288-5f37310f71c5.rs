use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::{HANDLE, HMODULE};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

fn call_get_module_file_name_ex_w() -> HRESULT {
    let mut filename = [0u16; 260];

    // Call GetModuleFileNameExW with None for both handles to get current module
    let result = unsafe { GetModuleFileNameExW(None::<HANDLE>, None::<HMODULE>, &mut filename) };

    if result == 0 {
        // Get the error code and convert to HRESULT
        let error = Error::from_thread();
        return error.code();
    }

    // Success - return S_OK
    HRESULT(0)
}
