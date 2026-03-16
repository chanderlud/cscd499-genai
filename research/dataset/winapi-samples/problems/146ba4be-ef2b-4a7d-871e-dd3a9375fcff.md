1) Create a ListBox and Add Items

**Spec:** Write a function that creates a ListBox control as a child window and populates it with string items using Win32 API messages.

**Constraints:**
- Use `CreateWindowExW` to create the ListBox with appropriate styles
- Use `SendMessageW` with `LB_ADDSTRING` to add each string item
- Handle string conversion to wide (UTF-16) format for Win32 API calls
- Return the ListBox window handle on success

**Signature:**
```rust
fn create_listbox(parent: HWND, instance: HINSTANCE, items: &[&str]) -> Result<HWND>
```

**Example:**
```rust
let listbox = create_listbox(hwnd, instance, &["Item 1", "Item 2", "Item 3"])?;
```