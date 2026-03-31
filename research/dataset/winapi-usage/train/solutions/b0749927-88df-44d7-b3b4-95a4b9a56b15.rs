use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::Com::{CoInitializeEx, COINIT};

fn call_co_initialize_ex() -> HRESULT {
    unsafe { CoInitializeEx(None, COINIT(0)) }
}
