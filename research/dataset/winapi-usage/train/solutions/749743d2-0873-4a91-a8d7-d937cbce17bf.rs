#![allow(dead_code, unused_imports)]

use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::Packaging::Appx::{
    AddPackageDependency, AddPackageDependencyOptions, PACKAGEDEPENDENCY_CONTEXT,
};

fn call_add_package_dependency() -> WIN32_ERROR {
    let mut context = PACKAGEDEPENDENCY_CONTEXT(std::ptr::null_mut());
    let result = unsafe {
        AddPackageDependency(
            PCWSTR::null(),
            0,
            AddPackageDependencyOptions(0),
            &mut context,
            None,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
