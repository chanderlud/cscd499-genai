1) Retrieve Desktop Monitor Bounds
**Spec:** Write a function that sets the process to DPI aware, obtains the desktop window handle, determines the monitor it is primarily on, and returns the monitor's bounding rectangle coordinates.
**Constraints:** Must use the Windows crate APIs: SetProcessDPIAware, GetDesktopWindow, MonitorFromWindow with MONITOR_DEFAULTTONEAREST, and GetMonitorInfoW. Must handle unsafe code and errors via Result. Must correctly initialize MONITORINFO with its size.
**Signature:**
```rust
fn get_desktop_monitor_bounds() -> windows::core::Result<(i32, i32, i32, i32)>
```
**Example:**
```rust
let (left, top, right, bottom) = get_desktop_monitor_bounds()?;
println!("Desktop monitor bounds: left={}, top={}, right={}, bottom={}", left, top, right, bottom);
println!("Width: {}, Height: {}", right - left, bottom - top);
```