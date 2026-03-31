use windows::core::HRESULT;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::ProcessStatus::EmptyWorkingSet;

fn call_empty_working_set() -> HRESULT {
    unsafe {
        match EmptyWorkingSet(HANDLE::default()) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
