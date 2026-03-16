use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE,
};

struct IconHandle(HICON);

impl Drop for IconHandle {
    fn drop(&mut self) {
        // SAFETY: We own the HICON and are responsible for destroying it
        unsafe { DestroyIcon(self.0) };
    }
}

fn load_icon_from_resource(resource_id: u16, width: i32, height: i32) -> Result<IconHandle> {
    // SAFETY: GetModuleHandleW with null returns current process handle
    let hmodule = unsafe { GetModuleHandleW(None)? };

    // Convert HMODULE to HINSTANCE (they are the same underlying type)
    let hinstance = HINSTANCE(hmodule.0);

    // SAFETY: LoadImageW is called with valid parameters
    let handle = unsafe {
        LoadImageW(
            Some(hinstance),
            PCWSTR(resource_id as *const u16),
            IMAGE_ICON,
            width,
            height,
            LR_DEFAULTSIZE,
        )?
    };

    Ok(IconHandle(HICON(handle.0)))
}

fn main() -> Result<()> {
    // Example: load an icon with resource ID 1 and default size
    let _icon = load_icon_from_resource(1, 0, 0)?;
    println!("Icon loaded successfully from resource.");
    Ok(())
}
