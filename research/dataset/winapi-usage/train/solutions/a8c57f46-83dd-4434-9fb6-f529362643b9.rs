use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Controls::{TaskDialogIndirect, TASKDIALOGCONFIG};

fn call_task_dialog_indirect() -> windows::Win32::Foundation::WIN32_ERROR {
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        ..Default::default()
    };

    match unsafe { TaskDialogIndirect(&config, None, None, None) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
