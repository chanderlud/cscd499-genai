6. Create hard link (Win32)

* **Spec:** Create a hard link at `link_path` pointing to `existing_file`.
* **Constraints:** Use `CreateHardLinkW`. Fail if the target is a directory.
* **Signature:**

```rust
pub fn create_hard_link(link_path: &Path, existing_file: &Path) -> Result<()>;
```

* **Example:**

```rust
create_hard_link(Path::new(r"C:\tmp\alias.txt"), Path::new(r"C:\tmp\real.txt"))?;
```
