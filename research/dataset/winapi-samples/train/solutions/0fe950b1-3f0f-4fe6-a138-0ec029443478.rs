use std::marker::PhantomData;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{S_FALSE, S_OK};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT};

/// Token representing an initialized COM apartment on the current thread.
/// Must be explicitly uninitialized via `uninit()`.
/// Not Send or Sync to prevent cross-thread usage.
pub struct ComToken {
    _not_send_sync: PhantomData<*const ()>,
}

impl ComToken {
    /// Uninitializes COM on the current thread.
    /// Must be called exactly once for each successful `com_init`.
    pub fn uninit(self) {
        // SAFETY: This is safe because:
        // 1. The token can only be created by `com_init` which ensures COM was initialized.
        // 2. The token is !Send/!Sync, so it must be used on the same thread.
        // 3. Each token corresponds to exactly one successful CoInitializeEx call.
        unsafe {
            CoUninitialize();
        }
    }
}

/// Initializes COM on the current thread with the specified threading model.
/// Returns a token that must be explicitly uninitialized via `uninit()`.
///
/// # Arguments
/// * `coinit` - The COINIT flags specifying the threading model (e.g., COINIT_MULTITHREADED.0).
///
/// # Errors
/// Returns an error if:
/// - Invalid flags are provided
/// - The threading model is incompatible with an already-initialized apartment
/// - Any other COM initialization failure occurs
pub fn com_init(coinit: i32) -> Result<ComToken> {
    let flags = COINIT(coinit);

    // SAFETY: This is safe because:
    // 1. We're calling CoInitializeEx with valid flags from the caller.
    // 2. The reserved parameter is null as required.
    // 3. We properly handle the HRESULT result.
    let hr = unsafe { CoInitializeEx(None, flags) };

    // Check for success (S_OK or S_FALSE)
    if hr == S_OK || hr == S_FALSE {
        Ok(ComToken {
            _not_send_sync: PhantomData,
        })
    } else {
        // Convert HRESULT to Error for proper error propagation
        Err(Error::from_hresult(hr))
    }
}
