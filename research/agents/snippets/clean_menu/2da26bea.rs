use std::ptr::null;

use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyMenu, GetCursorPos, InsertMenuW, SendMessageW, SetForegroundWindow,
    TrackPopupMenu, HMENU, MF_BYPOSITION, TPM_NONOTIFY, TPM_RETURNCMD, WM_INITMENUPOPUP,
};

pub enum MenuOptions {
    Exit,
}

struct Menu(HMENU);

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            // Use `if let Err(e) = DestroyMenu(self.0)` to avoid the extra block
            if let Err(e) = DestroyMenu(self.0) {
                eprintln!("Failed to destroy menu: {}", e);
            }
        }
    }
}

const MENU_EXIT: usize = 1;

pub fn show_menu(hwnd: HWND) -> Result<Option<MenuOptions>> {
    unsafe {
        // Create the popup menu and immediately wrap it in `Menu` for RAII
        let menu = Menu(CreatePopupMenu()?);

        // Convert the exit text to UTF-16 once and reuse the pointer
        let exit_text: Vec<u16> = "Exit".encode_utf16().chain(std::iter::once(0)).collect();
        InsertMenuW(
            menu.0,
            u32::MAX,
            MF_BYPOSITION,
            MENU_EXIT,
            windows::core::PCWSTR::from_raw(exit_text.as_ptr()),
        )?;

        // Use `if` instead of an explicit boolean comparison for clarity
        if SetForegroundWindow(hwnd).into() {
            SendMessageW(
                hwnd,
                WM_INITMENUPOPUP,
                Some(WPARAM(menu.0 .0 as _)),
                Some(LPARAM(0)),
            );
        } else {
            return Err(Error::from_thread());
        }

        let mut cursor_position = POINT::default();
        GetCursorPos(&mut cursor_position)?;

        // TrackPopupMenu returns BOOL, check if it's non-zero
        let cmd = TrackPopupMenu(
            menu.0,
            TPM_NONOTIFY | TPM_RETURNCMD,
            cursor_position.x,
            cursor_position.y,
            Some(0),
            hwnd,
            Some(null()),
        );

        if cmd.0 == 0 {
            return Err(Error::from_thread());
        }

        match cmd.0 as usize {
            MENU_EXIT => Ok(Some(MenuOptions::Exit)),
            _ => Ok(None),
        }
    }
}

fn main() {
    // Create a simple window handle for demonstration
    let hwnd = HWND::default();

    match show_menu(hwnd) {
        Ok(Some(MenuOptions::Exit)) => println!("Exit selected"),
        Ok(None) => println!("No menu item selected"),
        Err(e) => eprintln!("Error showing menu: {}", e),
    }
}
