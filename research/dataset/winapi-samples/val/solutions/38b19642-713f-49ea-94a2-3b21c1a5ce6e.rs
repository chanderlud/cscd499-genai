use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_FILENAME_EXCED_RANGE, HANDLE, HWND, MAX_PATH};
use windows::Win32::Security::WinTrust::{
    WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
    WINTRUST_DATA_UICONTEXT, WINTRUST_FILE_INFO, WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE,
    WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE,
};

fn wide_path_fixed(path: &Path) -> Result<[u16; MAX_PATH as usize]> {
    let os_str = path.as_os_str();
    let mut wide_path = [0u16; MAX_PATH as usize];
    let mut i = 0;

    for code_unit in os_str.encode_wide() {
        if i >= (MAX_PATH as usize) - 1 {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_FILENAME_EXCED_RANGE.0,
            )));
        }
        wide_path[i] = code_unit;
        i += 1;
    }
    wide_path[i] = 0;
    Ok(wide_path)
}

struct WinTrustGuard {
    data: *mut WINTRUST_DATA,
}

impl Drop for WinTrustGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.data.is_null() {
                (*self.data).dwStateAction = WTD_STATEACTION_CLOSE;
                let _ = WinVerifyTrust(
                    HWND::default(),
                    &WINTRUST_ACTION_GENERIC_VERIFY_V2 as *const _ as *mut _,
                    self.data as *mut std::ffi::c_void,
                );
            }
        }
    }
}

pub fn verify_authenticode(path: &Path) -> Result<()> {
    let wide_path = wide_path_fixed(path)?;

    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(wide_path.as_ptr()),
        hFile: HANDLE::default(),
        pgKnownSubject: std::ptr::null_mut(),
    };

    let mut data = WINTRUST_DATA {
        cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
        pPolicyCallbackData: std::ptr::null_mut(),
        pSIPClientData: std::ptr::null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &mut file_info,
        },
        dwStateAction: WTD_STATEACTION_VERIFY,
        hWVTStateData: HANDLE::default(),
        pwszURLReference: PWSTR::default(),
        dwProvFlags: WTD_CACHE_ONLY_URL_RETRIEVAL,
        dwUIContext: WINTRUST_DATA_UICONTEXT(0),
        pSignatureSettings: std::ptr::null_mut(),
    };

    let _guard = WinTrustGuard { data: &mut data };

    // SAFETY: We're calling WinVerifyTrust with properly initialized structures.
    // The guard ensures cleanup happens even if verification fails.
    unsafe {
        let hr = WinVerifyTrust(
            HWND::default(),
            &WINTRUST_ACTION_GENERIC_VERIFY_V2 as *const _ as *mut _,
            &mut data as *mut _ as *mut std::ffi::c_void,
        );

        if hr < 0 {
            return Err(Error::from_hresult(HRESULT(hr)));
        }
    }

    Ok(())
}
