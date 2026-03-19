use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCSTR, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, ERROR_PROC_NOT_FOUND, HMODULE, NTSTATUS};
use windows::Win32::Security::Cryptography::{
    BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE, BCRYPT_BLOCK_PADDING, BCRYPT_CHAINING_MODE,
    BCRYPT_CHAIN_MODE_CBC, BCRYPT_HANDLE, BCRYPT_KEY_HANDLE, BCRYPT_OBJECT_LENGTH,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

// Helper function to create wide null-terminated strings
fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

// Define function pointer types for BCrypt functions
type BCryptOpenAlgorithmProviderFn = unsafe extern "system" fn(
    phalgorithm: *mut BCRYPT_ALG_HANDLE,
    pszalgid: PCWSTR,
    pszimplementation: PCWSTR,
    dwflags: u32,
) -> NTSTATUS;

type BCryptSetPropertyFn = unsafe extern "system" fn(
    hobject: BCRYPT_HANDLE,
    pszproperty: PCWSTR,
    pbinput: *const u8,
    cbinput: u32,
    dwflags: u32,
) -> NTSTATUS;

type BCryptGetPropertyFn = unsafe extern "system" fn(
    hobject: BCRYPT_HANDLE,
    pszproperty: PCWSTR,
    pboutput: *mut u8,
    cboutput: u32,
    pcbresult: *mut u32,
    dwflags: u32,
) -> NTSTATUS;

type BCryptGenerateSymmetricKeyFn = unsafe extern "system" fn(
    halgorithm: BCRYPT_ALG_HANDLE,
    phkey: *mut BCRYPT_KEY_HANDLE,
    pbkeyobject: *mut u8,
    cbkeyobject: u32,
    pbsecret: *const u8,
    cbsecret: u32,
    dwflags: u32,
) -> NTSTATUS;

type BCryptEncryptFn = unsafe extern "system" fn(
    hkey: BCRYPT_KEY_HANDLE,
    pbinput: *const u8,
    cbinput: u32,
    ppaddinginfo: *const std::ffi::c_void,
    pbiv: *mut u8,
    cbiv: u32,
    pboutput: *mut u8,
    cboutput: u32,
    pcbresult: *mut u32,
    dwflags: u32,
) -> NTSTATUS;

type BCryptCloseAlgorithmProviderFn =
    unsafe extern "system" fn(halgorithm: BCRYPT_ALG_HANDLE, dwflags: u32) -> NTSTATUS;

type BCryptDestroyKeyFn = unsafe extern "system" fn(hkey: BCRYPT_KEY_HANDLE) -> NTSTATUS;

// Helper to convert NTSTATUS to Result
fn ntstatus_to_result(status: NTSTATUS) -> Result<()> {
    if status.0 >= 0 {
        Ok(())
    } else {
        Err(Error::from_hresult(HRESULT(status.0)))
    }
}

// Helper to load a function from a DLL
fn delay_load(dll_name: PCWSTR, func_name: PCSTR) -> Result<*const std::ffi::c_void> {
    unsafe {
        let h_module = LoadLibraryW(dll_name)?;
        if h_module.is_invalid() {
            return Err(Error::from_thread());
        }

        let proc = GetProcAddress(h_module, func_name);
        if proc.is_none() {
            let _ = FreeLibrary(h_module);
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_PROC_NOT_FOUND.0,
            )));
        }

        Ok(proc.unwrap() as *const std::ffi::c_void)
    }
}

pub fn aes_cbc_encrypt_dynamic(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) -> Result<Vec<u8>> {
    // Load BCrypt functions dynamically
    let open_alg_provider: BCryptOpenAlgorithmProviderFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptOpenAlgorithmProvider\0".as_ptr()),
        )?)
    };

    let set_property: BCryptSetPropertyFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptSetProperty\0".as_ptr()),
        )?)
    };

    let get_property: BCryptGetPropertyFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptGetProperty\0".as_ptr()),
        )?)
    };

    let generate_symmetric_key: BCryptGenerateSymmetricKeyFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptGenerateSymmetricKey\0".as_ptr()),
        )?)
    };

    let encrypt: BCryptEncryptFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptEncrypt\0".as_ptr()),
        )?)
    };

    let close_alg_provider: BCryptCloseAlgorithmProviderFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptCloseAlgorithmProvider\0".as_ptr()),
        )?)
    };

    let destroy_key: BCryptDestroyKeyFn = unsafe {
        std::mem::transmute(delay_load(
            PCWSTR(wide_null(OsStr::new("bcrypt.dll")).as_ptr()),
            PCSTR(b"BCryptDestroyKey\0".as_ptr()),
        )?)
    };

    // Open algorithm provider
    let mut h_alg = BCRYPT_ALG_HANDLE::default();
    let alg_id = BCRYPT_AES_ALGORITHM;
    let impl_name = wide_null(OsStr::new("Microsoft Primitive Provider"));

    let status = unsafe { open_alg_provider(&mut h_alg, alg_id, PCWSTR(impl_name.as_ptr()), 0) };
    ntstatus_to_result(status)?;

    // Set chaining mode to CBC
    let chaining_mode = BCRYPT_CHAINING_MODE;
    let cbc_mode = BCRYPT_CHAIN_MODE_CBC;

    let status = unsafe {
        set_property(
            BCRYPT_HANDLE(h_alg.0),
            chaining_mode,
            cbc_mode.as_ptr() as *const u8,
            (cbc_mode.len() * 2) as u32,
            0,
        )
    };

    // Clean up algorithm provider if SetProperty fails
    if let Err(e) = ntstatus_to_result(status) {
        unsafe { close_alg_provider(h_alg, 0) };
        return Err(e);
    }

    // Get object length for key object using BCryptGetProperty
    let mut object_length = 0u32;
    let mut result_length = 0u32;
    let obj_length_prop = BCRYPT_OBJECT_LENGTH;

    let status = unsafe {
        get_property(
            BCRYPT_HANDLE(h_alg.0),
            obj_length_prop,
            &mut object_length as *mut u32 as *mut u8,
            std::mem::size_of::<u32>() as u32,
            &mut result_length,
            0,
        )
    };
    ntstatus_to_result(status)?;

    // Generate symmetric key
    let mut h_key = BCRYPT_KEY_HANDLE::default();
    let mut key_object = vec![0u8; object_length as usize];

    let status = unsafe {
        generate_symmetric_key(
            h_alg,
            &mut h_key,
            key_object.as_mut_ptr(),
            object_length,
            key.as_ptr(),
            key.len() as u32,
            0,
        )
    };

    if let Err(e) = ntstatus_to_result(status) {
        unsafe { close_alg_provider(h_alg, 0) };
        return Err(e);
    }

    // Encrypt data
    let mut iv_copy = *iv;
    let mut ciphertext = vec![0u8; plaintext.len() + 16]; // Extra space for padding
    let mut result_size = 0u32;

    let status = unsafe {
        encrypt(
            h_key,
            plaintext.as_ptr(),
            plaintext.len() as u32,
            std::ptr::null(),
            iv_copy.as_mut_ptr(),
            iv_copy.len() as u32,
            ciphertext.as_mut_ptr(),
            ciphertext.len() as u32,
            &mut result_size,
            BCRYPT_BLOCK_PADDING.0, // Convert BCRYPT_FLAGS to u32
        )
    };

    // Clean up regardless of encryption result
    unsafe {
        destroy_key(h_key);
        close_alg_provider(h_alg, 0);
    }

    ntstatus_to_result(status)?;

    // Resize ciphertext to actual size
    ciphertext.truncate(result_size as usize);

    Ok(ciphertext)
}
