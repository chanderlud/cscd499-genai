**Spec:** Implement a function that copies the contents of one file to another using a fixed-size buffer (4096 bytes). The function should open the source file for reading with `FILE_GENERIC_READ`, create or overwrite the destination file with `CREATE_ALWAYS` and `FILE_GENERIC_WRITE`, then repeatedly read chunks from the source and write them to the destination until the entire file is copied. File handles must be closed via RAII (using a wrapper struct with `Drop` implementation) to ensure cleanup even on error.

**Constraints:**
- Use `CreateFileW` with `FILE_GENERIC_READ` for source and `FILE_GENERIC_WRITE` with `CREATE_ALWAYS` for destination
- Use `ReadFile` and `WriteFile` in a loop with a 4096-byte buffer
- Must handle files larger than the buffer size by looping until `ReadFile` returns 0 bytes read
- Ensure both file handles are properly closed via RAII (custom `FileHandle` struct implementing `Drop`)
- Verify that `bytes_written` equals `bytes_read` for each chunk, returning an error on mismatch
- Return `windows::core::Result<()>` indicating success or failure

**Signature:**
```rust
pub fn copy_file_chunked(src_path: &str, dst_path: &str) -> windows::core::Result<()>
```

**Example:**
```rust
// Copy a large file using 4KB chunks
copy_file_chunked("large_input.dat", "backup_copy.dat")?;
println!("File copied successfully!");
```