use windows::core::w;
use windows::core::{Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{
    BCryptAddContextFunction, BCRYPT_INTERFACE, BCRYPT_TABLE,
};

fn call_b_crypt_add_context_function() -> Result<NTSTATUS> {
    let table = BCRYPT_TABLE(0);
    let context = w!("TestContext");
    let interface = BCRYPT_INTERFACE(0);
    let function = w!("TestFunction");
    let position = 0u32;

    // SAFETY: All parameters are valid wide strings and u32 values.
    let status = unsafe { BCryptAddContextFunction(table, context, interface, function, position) };

    status.ok()?;
    Ok(status)
}
