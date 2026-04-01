use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Media::timeGetTime;

fn call_time_get_time() -> WIN32_ERROR {
    // timeGetTime returns u32 (milliseconds), not a Result or error code
    // Since there's no error to convert, we return a default WIN32_ERROR
    let _time: u32 = unsafe { timeGetTime() };
    WIN32_ERROR(0)
}
