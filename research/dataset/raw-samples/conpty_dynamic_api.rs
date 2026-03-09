use std::ffi::c_void;
use std::sync::Once;
use windows::core;
use windows::Win32::System::Console::{HPCON, COORD};
use windows::Win32::Foundation::{HANDLE, BOOL, WIN32_ERROR, HMODULE};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

type CreatePseudoConsoleFn = unsafe extern "system" fn(
    size: COORD,
    hInput: HANDLE,
    hOutput: HANDLE,
    dwFlags: u32,
    phPC: *mut HPCON,
) -> WIN32_ERROR;

type ResizePseudoConsoleFn = unsafe extern "system" fn(
    hPC: HPCON,
    size: COORD,
) -> WIN32_ERROR;

type ClosePseudoConsoleFn = unsafe extern "system" fn(
    hPC: HPCON,
) -> ();

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
    let kernel32 = match get_module_handle("kernel32.dll") {
        Ok(k) => k,
        Err(_) => {
            INITIALIZED = true;
            return;
        }
    };

    // Attempt to get function pointers for ConPTY APIs
    if let Some(func_ptr) = get_proc_address(kernel32, "CreatePseudoConsole") {
        CREATE_PSEUDOCONSOLE = Some(std::mem::transmute(func_ptr));
    }
    if let Some(func_ptr) = get_proc_address(kernel32, "ResizePseudoConsole") {
        RESIZE_PSEUDOCONSOLE = Some(std::mem::transmute(func_ptr));
    }
    if let Some(func_ptr) = get_proc_address(kernel32, "ClosePseudoConsole") {
        CLOSE_PSEUDOCONSOLE = Some(std::mem::transmute(func_ptr));
    }

    INITIALIZED = true;
}

unsafe fn get_module_handle(module_name: &str) -> Result<HMODULE, ()> {
    use std::ffi::CString;

    let module_name_cstr = CString::new(module_name).unwrap();
    match GetModuleHandleA(core::PCSTR(module_name_cstr.as_ptr() as *const u8)) {
        Ok(h) => Ok(h),
        Err(_) => Err(()),
    }
}

unsafe fn get_proc_address(module: HMODULE, proc_name: &str) -> Option<*const c_void> {
    use std::ffi::CString;

    let proc_name_cstr = match CString::new(proc_name) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let func_ptr = GetProcAddress(module, core::PCSTR(proc_name_cstr.as_ptr() as *const u8));
    func_ptr.map(|ptr| ptr as *const c_void)
}

/// Checks if ConPTY APIs are available on the system (Windows 10+)
pub fn is_conpty_available() -> bool {
    unsafe {
        INITIALIZER.call_once(|| {
            initialize_conpty_apis();
        });

        CREATE_PSEUDOCONSOLE.is_some() &&
            RESIZE_PSEUDOCONSOLE.is_some() &&
            CLOSE_PSEUDOCONSOLE.is_some()
    }
}

/// Calls CreatePseudoConsole if available, returns error if not available
pub unsafe fn create_pseudo_console(
    size: COORD,
    h_input: HANDLE,
    h_output: HANDLE,
    dw_flags: u32,
) -> Result<HPCON, ()> {
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
                Err(())
            }
        },
        None => Err(()),
    }
}

/// Calls ResizePseudoConsole if available, returns error if not available
pub unsafe fn resize_pseudo_console(
    h_pc: HPCON,
    size: COORD,
) -> Result<WIN32_ERROR, ()> {
    INITIALIZER.call_once(|| {
        initialize_conpty_apis();
    });

    match RESIZE_PSEUDOCONSOLE {
        Some(func) => Ok(func(h_pc, size)),
        None => Err(()),
    }
}

/// Calls ClosePseudoConsole if available, returns error if not available
pub unsafe fn close_pseudo_console(
    h_pc: HPCON,
) -> Result<BOOL, ()> {
    INITIALIZER.call_once(|| {
        initialize_conpty_apis();
    });

    match CLOSE_PSEUDOCONSOLE {
        Some(func) => {
            func(h_pc);
            Ok(BOOL(1)) // Return success value
        },
        None => Err(()),
    }
}