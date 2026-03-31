#![allow(dead_code)]
use windows::core::{Error, Result};
use windows::Win32::System::Memory::AddSecureMemoryCacheCallback;

fn call_add_secure_memory_cache_callback() -> windows::core::HRESULT {
    // SAFETY: Passing None is safe as it represents a null function pointer,
    // which the API accepts for the callback parameter.
    unsafe { AddSecureMemoryCacheCallback(None) }
        .map(|_| windows::core::HRESULT::default())
        .unwrap_or_else(|e| e.code())
}
