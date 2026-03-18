**Spec:**
- Retrieve the foreground window handle using `GetForegroundWindow`. If the handle is invalid, return an error.
- Obtain the default IME window handle using `ImmGetDefaultIMEWnd`. If the handle is invalid, return an error.
- Send a `WM_IME_CONTROL` message with `IMC_SETCONVERSIONMODE` to set the conversion mode, passing the conversion mode in the `WPARAM`.
- If `SendMessageA` returns `LRESULT(-1)`, return an error.
- Otherwise, return `Ok(())`.

**Constraints:**
- Must use `SendMessageA` to communicate with the IME window.
- The conversion mode value is passed as the `WPARAM` (the entire value, not just the low-order word).
- The function should handle the case where the foreground window has no associated IME by returning an error.

**Signature:**
```rust
pub fn set_conversion_mode(mode: u32) -> windows::core::Result<()>
```

**Example:**
```rust
// Set conversion mode to 0 (typically "native" or "alphabetic")
set_conversion_mode(0)?;
```