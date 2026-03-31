**Spec:** Write a function `call_abort_system_shutdown_a` that calls `AbortSystemShutdownA` with concrete parameter values and returns the expected wrapper result.

**Constraints:**
- Call `AbortSystemShutdownA` with concrete parameter values
- Return `windows::core::HRESULT` from the wrapper
- Convert any API error into `HRESULT` rather than panicking

**Signature:**
```rust
fn call_abort_system_shutdown_a() -> windows::core::HRESULT
```

**Example:**
```rust
let hr = call_abort_system_shutdown_a();
```
