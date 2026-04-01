use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinSock::EnumProtocolsW;

fn call_enum_protocols_w() -> WIN32_ERROR {
    let mut buffer_length: u32 = 0;

    let result = unsafe { EnumProtocolsW(None, std::ptr::null_mut(), &mut buffer_length) };

    if result == 0 {
        let error = Error::from_thread();
        WIN32_ERROR(error.code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
