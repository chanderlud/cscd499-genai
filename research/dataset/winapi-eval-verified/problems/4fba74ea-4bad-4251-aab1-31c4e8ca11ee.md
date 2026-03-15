## 11) Win32 Error Message: FormatMessageW

* **Spec:** Convert a Win32 error code (e.g. `ERROR_ACCESS_DENIED`) into a human-readable message string using `FormatMessageW`. Trim trailing whitespace/newlines and return it.
* **Constraints:**

    * Use `FormatMessageW` with `FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS`.
    * If using `FORMAT_MESSAGE_ALLOCATE_BUFFER`, free with `LocalFree`.
    * Must not use `std::io::Error::from_raw_os_error(err).to_string()` (that dodges the point).
* **Signature:**

  ```rust
  pub fn format_win32_error_message(error_code: u32) -> std::io::Result<String> {
      todo!()
  }
  ```
* **Example:**

  ```rust
  let msg = format_win32_error_message(5 /* ERROR_ACCESS_DENIED */)?;
  assert!(!msg.is_empty());
  ```
