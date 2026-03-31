use windows::core::{w, Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::BCryptAddContextFunctionProvider;

fn call_b_crypt_add_context_function_provider() -> Result<NTSTATUS> {
    // SAFETY: All parameters are correctly typed. String literals are null-terminated wide strings
    // compatible with PCWSTR, and numeric values are valid constants for the API.
    let status = unsafe {
        BCryptAddContextFunctionProvider(
            1u32,
            w!("SSL"),
            0u32,
            w!("AES"),
            w!("Microsoft Primitive Provider"),
            0u32,
        )
    };
    status.ok()?;
    Ok(status)
}
