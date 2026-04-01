use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::Controls::{
    TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOG_COMMON_BUTTON_FLAGS, TASKDIALOG_FLAGS,
};

fn call_task_dialog_indirect() -> Result<()> {
    let mut config = TASKDIALOGCONFIG::default();
    config.cbSize = std::mem::size_of::<TASKDIALOGCONFIG>() as u32;
    config.hwndParent = HWND::default();
    config.hInstance = HINSTANCE::default();
    config.dwFlags = TASKDIALOG_FLAGS(0);
    config.dwCommonButtons = TASKDIALOG_COMMON_BUTTON_FLAGS(0);
    config.pszWindowTitle = PCWSTR::null();
    config.pszMainInstruction = PCWSTR::null();
    config.pszContent = PCWSTR::null();
    config.pszFooter = PCWSTR::null();
    config.pButtons = std::ptr::null();
    config.cButtons = 0;
    config.pRadioButtons = std::ptr::null();
    config.cRadioButtons = 0;
    config.nDefaultButton = 0;
    config.nDefaultRadioButton = 0;
    config.lpCallbackData = 0;
    config.cxWidth = 0;

    unsafe { TaskDialogIndirect(&config, None, None, None) }
}
