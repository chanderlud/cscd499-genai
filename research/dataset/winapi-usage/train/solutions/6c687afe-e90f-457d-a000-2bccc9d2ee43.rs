use std::ffi::OsStr;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::Controls::{
    TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TASKDIALOGCONFIG_1,
    TASKDIALOG_COMMON_BUTTON_FLAGS, TASKDIALOG_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::HICON;

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}

fn call_task_dialog_indirect() -> Result<()> {
    let title = wide_null(OsStr::new("Task Dialog"));
    let instruction = wide_null(OsStr::new("Task Dialog"));
    let content = wide_null(OsStr::new("This is a task dialog."));
    let footer = wide_null(OsStr::new("Footer text"));

    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        hwndParent: HWND(std::ptr::null_mut()),
        hInstance: HINSTANCE(std::ptr::null_mut()),
        dwFlags: TASKDIALOG_FLAGS(0),
        dwCommonButtons: TASKDIALOG_COMMON_BUTTON_FLAGS(0),
        pszWindowTitle: PCWSTR::from_raw(title.as_ptr()),
        Anonymous1: TASKDIALOGCONFIG_0 {
            hMainIcon: HICON::default(),
        },
        pszMainInstruction: PCWSTR::from_raw(instruction.as_ptr()),
        pszContent: PCWSTR::from_raw(content.as_ptr()),
        pszFooter: PCWSTR::from_raw(footer.as_ptr()),
        pButtons: std::ptr::null(),
        cButtons: 0,
        pRadioButtons: std::ptr::null(),
        cRadioButtons: 0,
        nDefaultButton: 0,
        nDefaultRadioButton: 0,
        pszVerificationText: PCWSTR::null(),
        pszExpandedInformation: PCWSTR::null(),
        pszExpandedControlText: PCWSTR::null(),
        pszCollapsedControlText: PCWSTR::null(),
        pfCallback: None,
        lpCallbackData: 0,
        cxWidth: 0,
        Anonymous2: TASKDIALOGCONFIG_1 {
            hFooterIcon: HICON::default(),
        },
    };

    unsafe { TaskDialogIndirect(&config, None, None, None) }
}
