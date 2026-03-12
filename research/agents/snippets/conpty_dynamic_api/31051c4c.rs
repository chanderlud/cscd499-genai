use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Console::{COORD, HPCON};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

fn wide_null(s: &str) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(once(0))
        .collect()
}

fn main() -> Result<()> {
    // Load kernel32.dll dynamically using GetModuleHandleW with NULL
    let kernel32_module = unsafe { GetModuleHandleW(None)? };
    println!(
        "Successfully loaded kernel32.dll module handle: {:?}",
        kernel32_module
    );

    // Retrieve function pointers for ConPTY APIs using GetProcAddress
    let create_pseudo_console_fn = unsafe {
        let func_ptr = GetProcAddress(
            kernel32_module,
            windows::core::PCSTR(wide_null("CreatePseudoConsoleW").as_ptr() as *const u8),
        )
        .ok_or_else(Error::from_thread)?;
        Some(std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> WIN32_ERROR,
        >(func_ptr))
    };

    let resize_pseudo_console_fn = unsafe {
        let func_ptr = GetProcAddress(
            kernel32_module,
            windows::core::PCSTR(wide_null("ResizePseudoConsoleW").as_ptr() as *const u8),
        )
        .ok_or_else(Error::from_thread)?;
        Some(std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            unsafe extern "system" fn(HPCON, COORD) -> WIN32_ERROR,
        >(func_ptr))
    };

    let close_pseudo_console_fn = unsafe {
        let func_ptr = GetProcAddress(
            kernel32_module,
            windows::core::PCSTR(wide_null("ClosePseudoConsoleW").as_ptr() as *const u8),
        )
        .ok_or_else(Error::from_thread)?;
        Some(std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            unsafe extern "system" fn(HPCON) -> i32,
        >(func_ptr))
    };

    println!("Retrieved function pointers for ConPTY APIs");

    // Create pseudo-console using CreatePseudoConsoleW
    let input_handle = HANDLE(std::ptr::null_mut());
    let output_handle = HANDLE(std::ptr::null_mut());
    let size = COORD { X: 80, Y: 25 };
    let dw_flags = 0u32;

    let mut pseudo_console_handle = HPCON::default();

    if let Some(create_fn) = create_pseudo_console_fn {
        let result = unsafe {
            create_fn(
                size,
                input_handle,
                output_handle,
                dw_flags,
                &mut pseudo_console_handle,
            )
        };

        if result.0 == 0 {
            println!(
                "Successfully created pseudo-console with handle: {:?}",
                pseudo_console_handle
            );
        } else {
            println!("Failed to create pseudo-console: error code {}", result.0);
        }
    } else {
        println!("CreatePseudoConsoleW function not available");
    }

    // Resize pseudo-console using ResizePseudoConsoleW
    if let Some(resize_fn) = resize_pseudo_console_fn {
        let new_size = COORD { X: 120, Y: 30 };
        let result = unsafe { resize_fn(pseudo_console_handle, new_size) };

        if result.0 == 0 {
            println!(
                "Successfully resized pseudo-console to {:?}x{:?}",
                new_size.X, new_size.Y
            );
        } else {
            println!("Failed to resize pseudo-console: error code {}", result.0);
        }
    } else {
        println!("ResizePseudoConsoleW function not available");
    }

    // Close pseudo-console using ClosePseudoConsoleW
    if let Some(close_fn) = close_pseudo_console_fn {
        let result = unsafe { close_fn(pseudo_console_handle) };

        if result == 1 {
            println!("Successfully closed pseudo-console");
        } else {
            println!("Failed to close pseudo-console: error code {}", result);
        }
    } else {
        println!("ClosePseudoConsoleW function not available");
    }

    Ok(())
}
