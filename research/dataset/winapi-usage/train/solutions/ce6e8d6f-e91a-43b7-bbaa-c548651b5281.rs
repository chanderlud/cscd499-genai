use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Cryptography::BCryptAddContextFunctionProvider;

fn call_b_crypt_add_context_function_provider() -> WIN32_ERROR {
    let status = unsafe { BCryptAddContextFunctionProvider(0, None, 0, None, None, 0) };
    WIN32_ERROR(status.0 as u32)
}
