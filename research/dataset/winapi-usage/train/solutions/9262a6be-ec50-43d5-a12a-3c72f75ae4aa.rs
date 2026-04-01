use windows::core::Result;
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR};
use windows::Win32::System::Pipes::CreatePipe;

fn call_create_pipe() -> WIN32_ERROR {
    let mut read_pipe: HANDLE = HANDLE(std::ptr::null_mut());
    let mut write_pipe: HANDLE = HANDLE(std::ptr::null_mut());

    let result: Result<()> = unsafe { CreatePipe(&mut read_pipe, &mut write_pipe, None, 0) };

    match result {
        Ok(()) => ERROR_SUCCESS,
        Err(e) => {
            let hresult = e.code();
            let win32_code = hresult.0;
            WIN32_ERROR(win32_code.try_into().unwrap())
        }
    }
}
