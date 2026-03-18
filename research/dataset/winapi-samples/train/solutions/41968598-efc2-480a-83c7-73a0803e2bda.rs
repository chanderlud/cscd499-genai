use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
};

fn get_virtual_screen_size() -> (i32, i32) {
    // SAFETY: GetSystemMetrics is a safe Win32 API call with no preconditions
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    (width, height)
}
