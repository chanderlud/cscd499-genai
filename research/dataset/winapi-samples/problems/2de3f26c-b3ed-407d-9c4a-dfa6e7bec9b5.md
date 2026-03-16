1) Set and get selection range in an edit control

**Spec:** Write a function that creates an edit control with sample text, sets the selection to specified character ranges, and retrieves the current selection range.

**Constraints:**
- Use the Win32 API `SendMessageW` with `EM_SETSEL` and `EM_GETSEL` messages
- The function must create a window and edit control, set multiple selection ranges, and print the results
- Handle errors appropriately using `Result` types

**Signature:**
```rust
pub fn demonstrate_edit_selection() -> Result<()>
```

**Example:**
```rust
// Creates window with edit control, sets selections, and prints ranges
demonstrate_edit_selection()?;
// Output:
// Selection range: 0 to 5
// Updated selection range: 10 to 15
// Full selection range: 0 to 53
```