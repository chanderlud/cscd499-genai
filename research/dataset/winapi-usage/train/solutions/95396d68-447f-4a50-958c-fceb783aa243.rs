use windows::core::{Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{BCryptDecrypt, BCRYPT_FLAGS, BCRYPT_KEY_HANDLE};

fn call_b_crypt_decrypt() -> Result<NTSTATUS> {
    // Create a dummy key handle (null pointer for this example)
    let hkey = BCRYPT_KEY_HANDLE(std::ptr::null_mut());

    // Input data
    let pbinput: &[u8] = &[0u8; 16];

    // IV buffer
    let mut pbiv = [0u8; 16];

    // Output buffer
    let mut pboutput = [0u8; 128];

    // Result size
    let mut pcbresult = 0u32;

    // Flags
    let dwflags = BCRYPT_FLAGS(0);

    // Call BCryptDecrypt
    let ntstatus = unsafe {
        BCryptDecrypt(
            hkey,
            Some(pbinput),
            None,
            Some(&mut pbiv[..]),
            Some(&mut pboutput[..]),
            &mut pcbresult,
            dwflags,
        )
    };

    // Check NTSTATUS and construct Result manually
    // NTSTATUS >= 0 indicates success (STATUS_SUCCESS is 0)
    if ntstatus.0 >= 0 {
        Ok(ntstatus)
    } else {
        Err(Error::from_thread())
    }
}
