1) Get System Locale
**Spec:** Write a function that retrieves the user's default locale name as a string using the Windows API.
**Constraints:**
- Use the `GetUserDefaultLocaleName` function from the `windows` crate.
- The function must return a `Result<String>`.
- Use a fixed-size buffer of 85 `u16` elements (the maximum length for a locale name).
- Convert the UTF-16 buffer to a Rust `String`, excluding the null terminator.
- If the API call fails, return an error using `Error::from_thread()`.
**Signature:**
```rust
fn get_system_locale() -> Result<String>
```
**Example:**
```rust
fn main() -> Result<()> {
    let locale = get_system_locale()?;
    println!("System locale: {}", locale);
    Ok(())
}
```