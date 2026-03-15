use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyMenu, GetCursorPos, InsertMenuW, SendMessageW, SetForegroundWindow,
    TrackPopupMenu, HMENU, MF_BYPOSITION, TPM_NONOTIFY, TPM_RETURNCMD, WM_INITMENUPOPUP,
};

/// Helper to create a null-terminated wide string
fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

/// Application menu options that map to actions
pub enum MenuOptions {
    Exit,
    About,
    Settings,
}

/// RAII wrapper for popup menu to ensure cleanup
struct Menu(HMENU);

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe { let _ = DestroyMenu(self.0); }
    }
}

const MENU_EXIT: usize = 1;
const MENU_ABOUT: usize = 2;
const MENU_SETTINGS: usize = 3;

/// Creates and displays a popup menu at the current cursor position.
///
/// Returns the selected menu option if a selection was made, None otherwise.
pub fn show_menu(hwnd: HWND) -> Result<Option<MenuOptions>> {
    unsafe {
        // Create the popup menu
        let menu = Menu(CreatePopupMenu()?);

        // Insert menu items using W-suffix function
        let exit_str = wide_null(std::ffi::OsStr::new("Exit"));
        InsertMenuW(
            menu.0,
            u32::MAX,
            MF_BYPOSITION,
            MENU_EXIT,
            PCWSTR(exit_str.as_ptr()),
        )?;

        let about_str = wide_null(std::ffi::OsStr::new("About"));
        InsertMenuW(
            menu.0,
            u32::MAX,
            MF_BYPOSITION,
            MENU_ABOUT,
            PCWSTR(about_str.as_ptr()),
        )?;

        let settings_str = wide_null(std::ffi::OsStr::new("Settings"));
        InsertMenuW(
            menu.0,
            u32::MAX,
            MF_BYPOSITION,
            MENU_SETTINGS,
            PCWSTR(settings_str.as_ptr()),
        )?;

        // Bring the target window to foreground before showing menu
        if !SetForegroundWindow(hwnd).as_bool() {
            return Err(windows::core::Error::from_thread());
        }

        // Send WM_INITMENUPOPUP to initialize the menu
        SendMessageW(
            hwnd,
            WM_INITMENUPOPUP,
            Some(windows::Win32::Foundation::WPARAM(menu.0 .0 as _)),
            Some(windows::Win32::Foundation::LPARAM(0)),
        );

        // Get current cursor position for menu placement
        let mut cursor_position = POINT::default();
        GetCursorPos(&mut cursor_position)?;

        // Display the popup menu and get the selected command
        let cmd = TrackPopupMenu(
            menu.0,
            TPM_RETURNCMD | TPM_NONOTIFY,
            cursor_position.x,
            cursor_position.y,
            Some(0),
            hwnd,
            None,
        );

        // TrackPopupMenu returns BOOL; check if it succeeded and got a command
        if !cmd.as_bool() {
            return Err(windows::core::Error::from_thread());
        }

        // Map the command to a menu option
        match cmd.0 as usize {
            MENU_EXIT => Ok(Some(MenuOptions::Exit)),
            MENU_ABOUT => Ok(Some(MenuOptions::About)),
            MENU_SETTINGS => Ok(Some(MenuOptions::Settings)),
            _ => Ok(None),
        }
    }
}

/// Simulates an application action based on menu selection
fn handle_menu_action(action: MenuOptions) {
    match action {
        MenuOptions::Exit => {
            println!("Exiting application...");
            std::process::exit(0);
        }
        MenuOptions::About => {
            println!("About: Menu Selection Demo v1.0");
        }
        MenuOptions::Settings => {
            println!("Opening settings...");
        }
    }
}

fn main() -> Result<()> {
    // Simulate a window handle (in a real app, this would be the actual window handle)
    let hwnd = HWND::default();

    println!("Popup menu demo. Press Enter to show the menu, or Ctrl+C to exit.\n");

    // Show the menu and handle the selection
    loop {
        println!("Press Enter to show menu...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().is_empty() {
            match show_menu(hwnd)? {
                Some(action) => {
                    handle_menu_action(action);
                    println!();
                }
                None => {
                    println!("No menu selection made.\n");
                }
            }
        }
    }
}
