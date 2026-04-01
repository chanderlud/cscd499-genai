use windows::core::Result;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Pipes::CreatePipe;

fn call_create_pipe() -> Result<()> {
    let mut read_pipe: HANDLE = HANDLE(std::ptr::null_mut());
    let mut write_pipe: HANDLE = HANDLE(std::ptr::null_mut());

    unsafe {
        CreatePipe(&mut read_pipe, &mut write_pipe, None, 0);
    }

    Ok(())
}
