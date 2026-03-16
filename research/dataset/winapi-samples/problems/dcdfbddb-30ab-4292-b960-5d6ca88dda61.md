1) System Tray Icon with GUID and Version Configuration

**Spec:** Create a system tray notification icon using a builder pattern that supports setting a GUID for persistent identification and configuring the icon version to NOTIFYICON_VERSION_4.

**Constraints:**
- Must use the `windows` crate for Win32 API bindings
- Must create a hidden window to receive notification messages
- Must implement a builder pattern for configuring the NOTIFYICONDATAW structure
- Must support setting a GUID via NIF_GUID flag
- Must set the icon version using NIM_SETVERSION message
- Must properly clean up the icon on application exit

**Signature:**
```rust
struct NotifyIcon {
    data: NOTIFYICONDATAW,
}

impl NotifyIcon {
    fn new() -> Self;
    fn window_handle(self, handle: HWND) -> Self;
    fn tip(self, s: impl Into<String>) -> Self;
    fn icon(self, icon: HICON) -> Self;
    fn callback_message(self, callback_msg: u32) -> Self;
    fn guid(self, guid: impl Into<GUID>) -> Self;
    fn version(self, version: u32) -> Self;
    fn notify_add(&self) -> Result<()>;
    fn notify_delete(&self) -> Result<()>;
    fn notify_set_version(&self) -> Result<()>;
}
```

**Example:**
```rust
let icon_guid = GUID::from_values(0x12345678, 0x1234, 0x5678, [0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78]);
let notify_icon = NotifyIcon::new()
    .window_handle(hwnd)
    .tip("My Application")
    .icon(icon)
    .callback_message(WM_USER + 1)
    .guid(icon_guid)
    .version(NOTIFYICON_VERSION_4);
notify_icon.notify_add()?;
notify_icon.notify_set_version()?;
```