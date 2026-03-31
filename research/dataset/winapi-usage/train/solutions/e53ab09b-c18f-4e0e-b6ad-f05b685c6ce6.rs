use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Cryptography::{
    BCryptAddContextFunction, BCRYPT_INTERFACE, BCRYPT_TABLE,
};

fn call_b_crypt_add_context_function() -> WIN32_ERROR {
    let result = unsafe {
        BCryptAddContextFunction(
            BCRYPT_TABLE(0),
            w!("context"),
            BCRYPT_INTERFACE(0),
            w!("function"),
            0,
        )
    };

    match result.ok() {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
