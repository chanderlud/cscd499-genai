use windows::core::HRESULT;
use windows::Win32::Security::Cryptography::{BCryptCloseAlgorithmProvider, BCRYPT_ALG_HANDLE};

fn call_b_crypt_close_algorithm_provider() -> HRESULT {
    let result =
        unsafe { BCryptCloseAlgorithmProvider(BCRYPT_ALG_HANDLE(std::ptr::null_mut()), 0) };

    if result.is_ok() {
        HRESULT(0)
    } else {
        result.to_hresult()
    }
}
