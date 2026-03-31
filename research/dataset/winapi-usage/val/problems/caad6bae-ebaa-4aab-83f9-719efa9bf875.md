**Spec:** Write a function `call_authz_add_sids_to_context` that calls `AuthzAddSidsToContext` with concrete parameter values and returns the expected wrapper result.

**Constraints:**
- Call `AuthzAddSidsToContext` with concrete parameter values
- Handle the result as `windows::core::Result<windows_core::Result<()>>`
- Use `?` for error propagation

**Signature:**
```rust
fn call_authz_add_sids_to_context() -> windows::core::Result<windows_core::Result<()>>
```

**Example:**
```rust
let value = call_authz_add_sids_to_context()?;
```
