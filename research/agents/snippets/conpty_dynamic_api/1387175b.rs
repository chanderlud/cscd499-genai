use std::ffi::CString;
use std::sync::Once;
use windows::core::{Error, Result, BOOL, HRESULT};
use windows::Win32::Foundation::{HANDLE, HMODULE, WIN32_ERROR};
use windows::Win32::System::Console::{COORD, HPCON};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type CreatePseudoConsoleFn = unsafe extern "system" fn(
    size: COORD,
    h_input: HANDLE,
    h_output: HANDLE,
    dw_flags: u32,
    ph_pc: *mut HPCON,
) -> HRESULT;

type ResizePseudoConsoleFn = unsafe extern "system" fn(h_pc: HPCON, size: COORD) -> HRESULT;

type ClosePseudoConsoleFn = unsafe extern "system" fn(h_pc: HPCON) -> BOOL;

static mut CREATE_PSEUDOCONSOLE: Option<CreatePseudoConsoleFn> = None;
static mut RESIZE_PSEUDOCONSOLE: Option<ResizePseudoConsoleFn> = None;
static mut CLOSE_PSEUDOCONSOLE: Option<ClosePseudoConsoleFn> = None;
static mut INITIALIZED: bool = false;
static INITIALIZER: Once = Once::new();

unsafe fn initialize_conpty_apis() {
    if INITIALIZED {
        return;
    }

    // Load kernel32.dll which contains the ConPTY APIs
    let kernel32 =
        match GetModuleHandleA(windows::core::PCSTR("kernel32.dll\0".as_bytes().as_ptr())) {
            Ok(k) => k,
            Err(_) => {
                INITIALIZED = true;
                return;
            }
        };

    // Attempt to get function pointers for ConPTY APIs
    if let Some(func_ptr) = get_proc_address(kernel32, "CreatePseudoConsole") {
        CREATE_PSEUDOCONSOLE = Some(std::mem::transmute::<
            *const std::ffi::c_void,
            CreatePseudoConsoleFn,
        >(func_ptr));
    }
    if let Some(func_ptr) = get_proc_address(kernel32, "ResizePseudoConsole") {
        RESIZE_PSEUDOCONSOLE = Some(std::mem::transmute::<
            *const std::ffi::c_void,
            ResizePseudoConsoleFn,
        >(func_ptr));
    }
    if let Some(func_ptr) = get_proc_address(kernel32, "ClosePseudoConsole") {
        CLOSE_PSEUDOCONSOLE = Some(std::mem::transmute::<
            *const std::ffi::c_void,
            ClosePseudoConsoleFn,
        >(func_ptr));
    }

    INITIALIZED = true;
}

unsafe fn get_proc_address(module: HMODULE, proc_name: &str) -> Option<*const std::ffi::c_void> {
    let proc_name_cstr = match CString::new(proc_name) {
        Ok(s) => s,
        Err(_) => return None,
    };
    GetProcAddress(
        module,
        windows::core::PCSTR(proc_name_cstr.as_ptr() as *const u8),
    )
    .map(|ptr| ptr as *const std::ffi::c_void)
}

/// Checks if ConPTY APIs are available on the system (Windows 10+)
pub fn is_conpty_available() -> bool {
    unsafe {
        INITIALIZER.call_once(|| {
            initialize_conpty_apis();
        });

        // Safe to read after initialization
        CREATE_PSEUDOCONSOLE.is_some()
            && RESIZE_PSEUDOCONSOLE.is_some()
            && CLOSE_PSEUDOCONSOLE.is_some()
    }
}

/// Calls CreatePseudoConsole if available, returns error if not available
///
/// # Safety
///
/// This function is unsafe because it calls into Windows API functions that may have undefined behavior
/// if called with invalid parameters or in an invalid state. The caller must ensure that:
///
/// - The handles passed are valid and properly initialized
/// - The function is called in a context where Windows API calls are allowed
/// - The returned handle is properly managed and closed when no longer needed
pub unsafe fn create_pseudo_console(
    size: COORD,
    h_input: HANDLE,
    h_output: HANDLE,
    dw_flags: u32,
) -> Result<HPCON> {
    INITIALIZER.call_once(|| {
        initialize_conpty_apis();
    });

    match CREATE_PSEUDOCONSOLE {
        Some(func) => {
            let mut h_con: HPCON = HPCON::default();
            let result = func(size, h_input, h_output, dw_flags, &mut h_con);
            if result.is_ok() {
                Ok(h_con)
            } else {
                Err(Error::from_hresult(HRESULT(result.0)))
            }
        }
        None => Err(Error::from_hresult(HRESULT(0))),
    }
}

/// Calls ResizePseudoConsole if available, returns error if not available
///
/// # Safety
///
/// This function is unsafe because it calls into Windows API functions that may have undefined behavior
/// if called with invalid parameters or in an invalid state. The caller must ensure that:
///
/// - The pseudo-console handle is valid and was created by a previous successful call
/// - The function is called in a context where Windows API calls are allowed
pub unsafe fn resize_pseudo_console(h_pc: HPCON, size: COORD) -> Result<WIN32_ERROR> {
    INITIALIZER.call_once(|| {
        initialize_conpty_apis();
    });

    match RESIZE_PSEUDOCONSOLE {
        Some(func) => {
            let hr = func(h_pc, size);
            Ok(WIN32_ERROR(hr.0 as u32))
        }
        None => Err(Error::from_hresult(HRESULT(0))),
    }
}

/// Calls ClosePseudoConsole if available, returns error if not available
///
/// # Safety
///
/// This function is unsafe because it calls into Windows API functions that may have undefined behavior
/// if called with invalid parameters or in an invalid state. The caller must ensure that:
///
/// - The pseudo-console handle is valid and was created by a previous successful call
/// - The function is called in a context where Windows API calls are allowed
pub unsafe fn close_pseudo_console(h_pc: HPCON) -> Result<BOOL> {
    INITIALIZER.call_once(|| {
        initialize_conpty_apis();
    });

    match CLOSE_PSEUDOCONSOLE {
        Some(func) => {
            let result = func(h_pc);
            Ok(result)
        }
        None => Err(Error::from_hresult(HRESULT(0))),
    }
}

fn main() {
    println!("ConPTY Function Pointer Demo");
    println!("============================");

    // Check if ConPTY APIs are available
    if is_conpty_available() {
        println!("✓ ConPTY APIs are available on this system");

        // Create a pseudo-console with default size
        // Note: Using null handles for demonstration purposes
        let size = COORD { X: 80, Y: 24 };
        let h_input = HANDLE::default();
        let h_output = HANDLE::default();
        let dw_flags = 0u32;

        match unsafe { create_pseudo_console(size, h_input, h_output, dw_flags) } {
            Ok(h_pc) => {
                println!("✓ Successfully created pseudo-console handle: {:?}", h_pc);

                // Resize the pseudo-console
                let new_size = COORD { X: 120, Y: 30 };
                match unsafe { resize_pseudo_console(h_pc, new_size) } {
                    Ok(_) => println!("✓ Successfully resized pseudo-console"),
                    Err(e) => println!("✗ Failed to resize pseudo-console: {:?}", e),
                }

                // Close the pseudo-console
                match unsafe { close_pseudo_console(h_pc) } {
                    Ok(_) => println!("✓ Successfully closed pseudo-console"),
                    Err(e) => println!("✗ Failed to close pseudo-console: {:?}", e),
                }
            }
            Err(e) => {
                println!("✗ Failed to create pseudo-console: {:?}", e);
            }
        }
    } else {
        println!("✗ ConPTY APIs are not available on this system");
        println!("  (Requires Windows 10 or later)");
    }
}
