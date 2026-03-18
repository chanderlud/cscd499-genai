1) Create a Simple Win32 Window

**Spec:** Write a Rust program that creates a basic Win32 window and runs a message loop. The window must handle `WM_PAINT` by printing "WM_PAINT" and validating the client area, and handle `WM_DESTROY` by printing "WM_DESTROY" and posting a quit message.

**Constraints:**
- Use the `windows` crate for Win32 bindings
- Register a window class with `RegisterClassA`
- Create the window with `CreateWindowExA` using `WS_OVERLAPPEDWINDOW | WS_VISIBLE`
- Implement an `extern "system"` window procedure function
- Use `GetMessageA`/`DispatchMessageA` for the message loop

**Signature:**
```rust
fn main() -> windows::core::Result<()>
```

**Example:**
```rust
// Running the program creates a visible window titled "This is a sample window"
// Console output when window is painted: "WM_PAINT"
// Console output when window is closed: "WM_DESTROY"
```