6. Private object namespace + boundary descriptor

* **Spec:** Create a private namespace scoped to the current user, create a named mutex inside it, then open the same mutex by name and return whether it succeeded.
* **Constraints:** `CreateBoundaryDescriptorW`, `AddSIDToBoundaryDescriptor`, `CreatePrivateNamespaceW`, `CreateMutexW`, `OpenMutexW`, `ClosePrivateNamespace`.
* **Signature:**

```rust
pub fn private_namespace_mutex_roundtrip(ns_name: &str, mutex_name: &str) -> Result<bool>;
```

* **Example:**

```rust
let ok = private_namespace_mutex_roundtrip("MyNs", "MyMutex")?;
```
