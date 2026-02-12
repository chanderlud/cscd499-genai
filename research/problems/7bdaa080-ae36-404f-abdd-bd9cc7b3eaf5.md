49) WinRT: Write file into LocalFolder (ApplicationData::Current().LocalFolder())

* **Spec:** Create/replace a file in LocalFolder and write bytes.
* **Constraints:** Async WinRT APIs; you’ll need an async runtime or manual blocking.
* **Signature:**

```rust
pub async fn appdata_write_bytes(name: &str, data: &[u8]) -> Result<()>;
```

* **Example:**

```rust
appdata_write_bytes("cache.bin", b"\x01\x02").await?;
```