N) Remove ListBox Item

**Spec:** Write a Win32 application that creates a window containing a listbox control, populates it with four string items, and removes the second item (index 1) from the listbox.

**Constraints:**
- Use the `windows` crate for Win32 API bindings
- Create a main window with a listbox child control
- Add four string items to the listbox using `LB_ADDSTRING`
- Remove the item at index 1 using `LB_DELETESTRING`
- Handle the standard Windows message loop
- Print a success message to console after removal

**Signature:**
```rust
fn main() -> windows::core::Result<()>
```

**Example:**
```rust
// After running the program, the console will output:
// Successfully removed item at index 1
// The window will display a listbox containing:
// First Item
// Third Item
// Fourth Item
```