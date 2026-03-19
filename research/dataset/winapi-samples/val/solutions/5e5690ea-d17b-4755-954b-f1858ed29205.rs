use windows::{core::*, Win32::UI::WindowsAndMessaging::*};

fn main() {
    unsafe {
        let result = MessageBoxW(
            None,
            w!("Continue to next message?"),
            w!("Question"),
            MB_YESNO,
        );
        if result == IDYES {
            MessageBoxW(None, w!("You chose to continue"), w!("Wide Message"), MB_OK);
        } else {
            MessageBoxA(None, s!("You chose to stop"), s!("Ansi Message"), MB_OK);
        }
    }
}
