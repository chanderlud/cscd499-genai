1) List Running Processes

**Spec:** Write a function that creates a snapshot of all running processes using the Tool Help API and prints each process's ID, executable name, and parent process ID.

**Constraints:**
- Must use `CreateToolhelp32Snapshot` with `TH32CS_SNAPPROCESS`
- Must iterate using `Process32FirstW` and `Process32NextW`
- Must properly initialize `PROCESSENTRY32W.dwSize`
- Must handle `ERROR_NO_MORE_FILES` to terminate iteration
- Must close the snapshot handle with `CloseHandle`

**Signature:**
```rust
fn list_processes() -> windows::core::Result<()>
```

**Example:**
```rust
fn main() -> windows::core::Result<()> {
    list_processes()
}
```