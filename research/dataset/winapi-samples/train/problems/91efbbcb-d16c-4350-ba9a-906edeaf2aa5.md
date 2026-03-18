**Spec:** Write a function that accepts a WinRT `IDirect3DDevice` and returns the description string of the underlying DXGI adapter.
**Constraints:** Use the `windows` crate. The function must obtain the DXGI device by casting the `IDirect3DDevice` to `IDXGIDevice` (using `Interface::cast`), then get the adapter via `IDXGIDevice::GetAdapter` (unsafe) and the description via `IDXGIAdapter::GetDesc` (unsafe). Convert the adapter's `Description` field (a `[u16; 128]` array) to a Rust `String`.
**Signature:**
```rust
pub fn get_adapter_description(device: &IDirect3DDevice) -> Result<String>
```
**Example:**
```rust
let d3d_device = create_d3d_device()?;
let direct3d_device = create_direct3d_device(&d3d_device)?;
let description = get_adapter_description(&direct3d_device)?;
println!("Adapter description: {}", description);
```