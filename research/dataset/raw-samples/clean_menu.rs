use std::ptr::null;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        CreatePopupMenu, DestroyMenu, GetCursorPos, InsertMenuW, SendMessageW, SetForegroundWindow,
        TrackPopupMenu, HMENU, MF_BYPOSITION, TPM_NONOTIFY, TPM_RETURNCMD, WM_INITMENUPOPUP,
    },
};

pub enum MenuOptions {
    Exit,
}

struct Menu(HMENU);

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            DestroyMenu(self.0);
        }
    }
}

const MENU_EXIT: usize = 1;

pub fn show_menu(hwnd: HWND) -> Option<MenuOptions> {
    unsafe {
        let menu = Menu(CreatePopupMenu());

        InsertMenuW(menu.0, u32::MAX, MF_BYPOSITION, MENU_EXIT, "Exit");

        SetForegroundWindow(hwnd);
        SendMessageW(hwnd, WM_INITMENUPOPUP, WPARAM(menu.0 .0 as _), LPARAM(0));

        let mut cursor_position = Default::default();
        if GetCursorPos(&mut cursor_position) == false {
            return None;
        }

        let cmd = TrackPopupMenu(
            menu.0,
            TPM_RETURNCMD | TPM_NONOTIFY,
            cursor_position.x,
            cursor_position.y,
            0,
            hwnd,
            null(),
        );

        match cmd.0 as usize {
            MENU_EXIT => Some(MenuOptions::Exit),
            _ => None,
        }
    }
}