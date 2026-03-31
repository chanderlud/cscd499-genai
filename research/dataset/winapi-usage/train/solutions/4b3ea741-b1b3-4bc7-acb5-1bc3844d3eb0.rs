use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::RestartManager::RmEndSession;

fn call_rm_end_session() -> HRESULT {
    // SAFETY: RmEndSession is an unsafe Win32 API; we pass a concrete dummy session handle.
    let win32_err = unsafe { RmEndSession(0) };
    win32_err.to_hresult()
}
