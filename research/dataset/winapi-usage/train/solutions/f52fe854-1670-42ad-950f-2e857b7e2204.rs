use windows::core::{w, HRESULT};
use windows::Win32::Security::Cryptography::{
    BCryptAddContextFunction, BCRYPT_INTERFACE, BCRYPT_TABLE,
};

fn call_b_crypt_add_context_function() -> HRESULT {
    let status = unsafe {
        BCryptAddContextFunction(
            BCRYPT_TABLE(0),
            w!("context"),
            BCRYPT_INTERFACE(0),
            w!("function"),
            0,
        )
    };
    status.into()
}
