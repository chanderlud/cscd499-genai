Get Network Interface Statistics

**Spec:** Write a function that retrieves statistics for all active network interfaces using the Windows IP Helper API. The function should return a vector of strings, each containing the interface alias (name) and its received (RX) and transmitted (TX) byte counts.

**Constraints:**
- Use the `GetIfTable2` function to retrieve the interface table.
- Only include interfaces with an operational status of `IfOperStatusUp`.
- Convert the interface alias from UTF-16 to a Rust `String`, trimming any trailing null characters.
- Ensure the allocated table is properly freed using `FreeMibTable`.
- Handle potential errors from the Win32 API calls and return a `Result` type.

**Signature:**
```rust
fn get_interface_stats() -> Result<Vec<String>>
```

**Example:**
```rust
let interfaces = get_interface_stats()?;
for interface in interfaces {
    println!("{}", interface);
}
// Example output: "Ethernet: RX=12345 bytes, TX=67890 bytes"
```