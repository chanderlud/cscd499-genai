use windows::core::Result;
use windows::Win32::Networking::WinHttp::WinHttpCheckPlatform;

fn call_win_http_check_platform() -> Result<Result<()>> {
    unsafe { Ok(WinHttpCheckPlatform()) }
}
