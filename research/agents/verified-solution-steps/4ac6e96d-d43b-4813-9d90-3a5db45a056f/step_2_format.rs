use windows::core::{Error, Result};
use windows::Win32::Foundation::{S_FALSE, S_OK};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT};

pub struct ComGuard {
    _private: std::marker::PhantomData<*const ()>,
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        // SAFETY: CoUninitialize must be called exactly once for each successful
        // CoInitializeEx call. This is guaranteed by the RAII pattern.
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn com_init(coinit: i32) -> Result<ComGuard> {
    // SAFETY: CoInitializeEx is a valid Win32 API call. We pass the provided flags
    // and check the result. The function is safe to call as long as we handle the
    // return value properly.
    let hr = unsafe { CoInitializeEx(None, COINIT(coinit)) };

    // Check for success codes: S_OK (0) and S_FALSE (1)
    if hr == S_OK || hr == S_FALSE {
        Ok(ComGuard {
            _private: std::marker::PhantomData,
        })
    } else {
        // Convert the HRESULT to a windows::core::Error
        Err(Error::from_hresult(hr))
    }
}