1) Create a Win32 Window

**Spec:** Write a program that creates a Win32 window and runs a standard message loop until the window is closed.

**Constraints:**
- Use the `windows` crate for Win32 bindings
- Register a window class with a window procedure that handles `WM_DESTROY` by posting a quit message
- Create a window with `WS_OVERLAPPEDWINDOW` style and default position/size
- Show the window and run a message loop using `GetMessageW`, `TranslateMessage`, and `DispatchMessageW`

**Signature:**
```rust
fn main() -> windows::core::Result<()>
```

**Example:**
```rust
// Running the program should display a window titled "Sample Window"
// Closing the window should exit the program cleanly
```