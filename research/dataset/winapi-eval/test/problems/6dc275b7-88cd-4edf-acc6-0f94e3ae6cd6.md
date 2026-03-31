- **Spec:** Implement `make_file_read_only` so it marks the file at the given path as read-only by calling `SetFileAttributesW` with `FILE_ATTRIBUTE_READONLY`. Return `Ok(())` on success and the Windows error on failure.

- **Constraints:**
  - Use only the `windows` crate v0.62.2.
  - Call exactly one Windows API function: `SetFileAttributesW`.
  - Use `FILE_ATTRIBUTE_READONLY` as the attribute value.
  - Do not use `std::fs::set_permissions` inside the solution function.
  - The function must be deterministic and unit testable by checking the file's read-only state afterward.

- **Signature:**
  ```rust
  pub fn make_file_read_only(path: &str) -> windows::core::Result<()>;
````

* **Example:**

  ```rust
  fn main() -> windows::core::Result<()> {
      make_file_read_only(r"C:\temp\demo.txt")?;
      Ok(())
  }
  ```
