**Spec:** Write a function that imports a raw AES symmetric key into CNG using `BCryptImportKey`. The function should accept an algorithm handle for AES and a byte slice containing the raw key material (16, 24, or 32 bytes for AES-128, AES-192, or AES-256). It must construct the proper `BCRYPT_KEY_DATA_BLOB` header, import the key, and return an owned key handle.

**Constraints:**
- The key material must be exactly 16, 24, or 32 bytes long.
- The algorithm handle must be for the AES algorithm (opened via `BCryptOpenAlgorithmProvider`).
- The function must use `BCryptImportKey` (not `BCryptImportKeyPair`).
- The returned key handle must be wrapped in `Owned<BCRYPT_KEY_HANDLE>` for automatic resource cleanup.
- Invalid key lengths must return an error with `E_INVALIDARG`.

**Signature:**
```rust
pub fn import_aes_key(
    alg_handle: BCRYPT_ALG_HANDLE,
    key_data: &[u8],
) -> windows::core::Result<windows::core::Owned<BCRYPT_KEY_HANDLE>>
```

**Example:**
```rust
// Assume `aes_handle` is a valid BCRYPT_ALG_HANDLE for AES
let key_bytes = [0x42u8; 16]; // 128-bit AES key
let aes_key = import_aes_key(aes_handle, &key_bytes)?;
// Use aes_key for encryption/decryption operations
```