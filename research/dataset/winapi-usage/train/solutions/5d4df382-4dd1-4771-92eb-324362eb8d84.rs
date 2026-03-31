use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Security::Cryptography::BCryptAddContextFunctionProvider;

fn call_b_crypt_add_context_function_provider() -> HRESULT {
    let status = unsafe {
        BCryptAddContextFunctionProvider(0, PCWSTR::null(), 0, PCWSTR::null(), PCWSTR::null(), 0)
    };
    HRESULT::from(status)
}
