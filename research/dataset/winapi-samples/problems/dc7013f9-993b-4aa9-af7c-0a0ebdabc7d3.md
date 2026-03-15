1) Adjust Window Rectangle for Non-Client Area with DPI Awareness

**Spec:** Write a function that adjusts a client rectangle to the full window rectangle (including non-client area) for a given window handle, style, extended style, and DPI. The function must use the DPI-aware version of the adjustment API if available, otherwise fall back to the non-DPI-aware version.

**Constraints:**
- The function must check for the presence of a menu in the window using `GetMenu`.
- The function must dynamically load `AdjustWindowRectExForDpi` from user32.dll and cache the function pointer in a static `OnceLock`.
- If the DPI-aware function is not available, fall back to `AdjustWindowRectEx`.
- The function must return a `Result<RECT>`.

**Signature:**
```rust
pub fn adjust_window_rect_with_dpi(
    hwnd: HWND,
    rect: RECT,
    style: WINDOW_STYLE,
    style_ex: WINDOW_EX_STYLE,
    dpi: u32,
) -> Result<RECT>
```

**Example:**
```rust
let hwnd = unsafe { GetDesktopWindow() };
let client_rect = RECT { left: 100, top: 100, right: 900, bottom: 700 };
let style = WINDOW_STYLE(unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32);
let style_ex = WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32);
let full_rect = adjust_window_rect_with_dpi(hwnd, client_rect, style, style_ex, 96)?;
```