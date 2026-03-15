use windows::{
    core::{w, Error, Result, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            GetPropW, PostQuitMessage, RegisterClassExW, SetPropW, TranslateMessage,
            UnregisterClassW, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WNDCLASSEXW,
        },
    },
};

fn main() -> Result<()> {
    let class_name = w!("SetPropExampleClass");

    // SAFETY: GetModuleHandleW is safe to call with None
    let hinstance = unsafe { GetModuleHandleW(None)? };

    let wnd_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance.into(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    // SAFETY: RegisterClassExW is safe with valid WNDCLASSEXW
    let atom = unsafe { RegisterClassExW(&wnd_class) };
    if atom == 0 {
        return Err(Error::from_thread());
    }

    // SAFETY: CreateWindowExW is safe with valid parameters
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("SetProp Example"),
            WINDOW_STYLE::default(),
            100,
            100,
            400,
            300,
            None,
            None,
            Some(hinstance.into()),
            None,
        )?
    };

    // Set a property on the window using a static value to avoid dangling pointer
    let property_name = w!("ExampleProperty");
    static PROPERTY_VALUE: i32 = 42;
    let property_handle = HANDLE(&PROPERTY_VALUE as *const _ as _);

    // SAFETY: We're passing a valid window handle and property name
    unsafe { SetPropW(hwnd, PCWSTR(property_name.as_ptr()), Some(property_handle))? };

    // Retrieve the property to verify it was set
    // SAFETY: GetPropW is safe with valid window handle and property name
    let retrieved_handle = unsafe { GetPropW(hwnd, PCWSTR(property_name.as_ptr())) };

    if retrieved_handle.0.is_null() {
        return Err(Error::from_thread());
    }

    // SAFETY: We know the handle points to our static i32 value
    let retrieved_value = unsafe { *(retrieved_handle.0 as *const i32) };
    println!("Retrieved property value: {}", retrieved_value);

    // Run message loop
    let mut msg = MSG::default();
    // SAFETY: GetMessageW is safe with valid MSG pointer
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        // SAFETY: TranslateMessage and DispatchMessageW are safe with valid MSG
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Clean up
    // SAFETY: DestroyWindow is safe with valid window handle
    unsafe { DestroyWindow(hwnd)? };
    // SAFETY: UnregisterClassW is safe with valid class name and instance
    unsafe { UnregisterClassW(class_name, Some(hinstance.into()))? };

    Ok(())
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            // SAFETY: PostQuitMessage is a simple Win32 call
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: DefWindowProcW is safe with valid parameters
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
