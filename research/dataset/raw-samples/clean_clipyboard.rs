use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::size_of,
    os::raw::c_char,
};

use windows::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
            OpenClipboard, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    },
};

use crate::Win32Result;

const CF_OEMTEXT: u32 = 7;

pub struct Clipboard(PhantomData<HWND>);

impl Clipboard {
    pub fn has_text() -> bool {
        unsafe { IsClipboardFormatAvailable(CF_OEMTEXT) }.as_bool()
    }

    pub fn from_handle(hwnd: HWND) -> Win32Result<Self> {
        unsafe { OpenClipboard(hwnd) }.ok()?;

        Ok(Self(PhantomData))
    }

    pub fn get_contents(&self) -> Win32Result<String> {
        let content = unsafe { GetClipboardData(CF_OEMTEXT) }.ok()?;
        let lpwstr = unsafe { GlobalLock(content.0) };

        if lpwstr.is_null() {
            unsafe { GlobalUnlock(content.0) };

            return Err(windows::core::Error::from_win32());
        }

        let c_str = unsafe { CStr::from_ptr(lpwstr as *const c_char) };
        let text = c_str.to_string_lossy().into_owned();

        unsafe { GlobalUnlock(content.0) };

        Ok(text)
    }

    pub fn set_contents(&self, content: &str) -> Win32Result<bool> {
        let len = content.len() + 1;

        match CString::new(content) {
            Ok(text) => unsafe {
                let handle = GlobalAlloc(GMEM_MOVEABLE, len * size_of::<std::os::raw::c_char>());
                let handle = windows::Win32::Foundation::HANDLE(handle);
                let ptr = GlobalLock(handle.0);

                std::ptr::copy_nonoverlapping(text.as_ptr(), ptr as *mut c_char, len);

                GlobalUnlock(handle.0);
                EmptyClipboard().ok()?;
                SetClipboardData(CF_OEMTEXT, handle).ok()?;

                Ok(true)
            },
            Err(_) => Ok(false),
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}