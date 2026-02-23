#[cfg(all(test, windows))]
mod tests {
    use super::*;

    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use windows::core::{Error, PCWSTR};
    use windows::Win32::Foundation::{CloseHandle, HANDLE, FALSE};
    use windows::Win32::System::Memory::{
        MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE,
    };

    fn to_wide_null(s: &str) -> Vec<u16> {
        // Note: If `s` contains '\0', this will embed a NUL in the middle. That’s intentional for
        // the "reject NUL in name" test; Win32 would treat it as a terminator.
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn unique_mapping_name(tag: &str) -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let pid = std::process::id();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        // Use Local\ to avoid session/global ACL weirdness.
        format!(r"Local\create_named_mapping_test_{}_{}_{}_{}", tag, pid, nanos, n)
    }

    fn owned_handle_as_handle(h: &std::os::windows::io::OwnedHandle) -> HANDLE {
        HANDLE(h.as_raw_handle() as isize)
    }

    unsafe fn open_mapping(name: &str) -> windows::core::Result<HANDLE> {
        let wname = to_wide_null(name);
        let h = OpenFileMappingW(
            FILE_MAP_READ | FILE_MAP_WRITE,
            FALSE,
            PCWSTR(wname.as_ptr()),
        );
        if h.0 == 0 {
            Err(Error::from_win32())
        } else {
            Ok(h)
        }
    }

    unsafe fn map_view(h: HANDLE, bytes: usize) -> windows::core::Result<*mut u8> {
        let p = MapViewOfFile(h, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, bytes);
        if p.is_null() {
            Err(Error::from_win32())
        } else {
            Ok(p as *mut u8)
        }
    }

    unsafe fn unmap_view(p: *mut u8) {
        if !p.is_null() {
            let _ = UnmapViewOfFile(p as _);
        }
    }

    #[test]
    fn create_named_mapping_creates_and_is_openable_and_shared() -> windows::core::Result<()> {
        let name = unique_mapping_name("basic");
        let size = 4096;

        let h1 = create_named_mapping(&name, size).expect("create_named_mapping should succeed");
        let h1h = owned_handle_as_handle(&h1);

        // Verify it exists system-wide by opening it again by name.
        let h2 = unsafe { open_mapping(&name) }?;

        // Map from both handles to prove they refer to the same shared memory.
        let p1 = unsafe { map_view(h1h, size) }?;
        let p2 = unsafe { map_view(h2, size) }?;

        unsafe {
            // Write a pattern via p1, read via p2.
            let s1 = std::slice::from_raw_parts_mut(p1, size);
            let s2 = std::slice::from_raw_parts_mut(p2, size);

            s1[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
            s1[size - 1] = 0xEE;

            assert_eq!(&s2[0..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
            assert_eq!(s2[size - 1], 0xEE);

            unmap_view(p2);
            unmap_view(p1);
            let _ = CloseHandle(h2);
        }

        Ok(())
    }

    #[test]
    fn create_named_mapping_name_collision_errors_or_refers_to_same_mapping() -> windows::core::Result<()> {
        let name = unique_mapping_name("collision");
        let size = 4096;

        let h1 = create_named_mapping(&name, size).expect("first create should succeed");
        let h1h = owned_handle_as_handle(&h1);

        let p1 = unsafe { map_view(h1h, size) }?;
        unsafe {
            let s1 = std::slice::from_raw_parts_mut(p1, size);
            s1[0] = 0xAB;
        }

        match create_named_mapping(&name, size) {
            Ok(h2) => {
                // If your implementation allows collision, it MUST be the same underlying mapping.
                let h2h = owned_handle_as_handle(&h2);
                let p2 = unsafe { map_view(h2h, size) }?;

                unsafe {
                    let s2 = std::slice::from_raw_parts_mut(p2, size);
                    assert_eq!(s2[0], 0xAB, "second handle should see data from first");

                    s2[0] = 0xCD;

                    let s1 = std::slice::from_raw_parts_mut(p1, size);
                    assert_eq!(s1[0], 0xCD, "writes via second handle should reflect in first");

                    unmap_view(p2);
                }
            }
            Err(_e) => {
                // If your implementation rejects collisions, the original mapping should still
                // be openable by name and contain what we wrote.
                let hop = unsafe { open_mapping(&name) }?;
                let p = unsafe { map_view(hop, size) }?;
                unsafe {
                    let s = std::slice::from_raw_parts_mut(p, size);
                    assert_eq!(s[0], 0xAB);
                    unmap_view(p);
                    let _ = CloseHandle(hop);
                }
            }
        }

        unsafe { unmap_view(p1) };
        Ok(())
    }

    #[test]
    fn create_named_mapping_rejects_zero_size() {
        let name = unique_mapping_name("zero_size");
        let r = create_named_mapping(&name, 0);
        assert!(r.is_err(), "size=0 should be rejected for pagefile-backed shared memory");
    }

    #[test]
    fn create_named_mapping_rejects_nul_in_name() {
        // Embedded NUL would truncate the Win32 name; the function should treat this as invalid input.
        let name = format!(r"Local\bad_name\0{}", unique_mapping_name("nul"));
        let r = create_named_mapping(&name, 4096);
        assert!(r.is_err(), "names containing NUL should be rejected");
    }

    #[test]
    fn create_named_mapping_small_size_maps_and_rw() -> windows::core::Result<()> {
        let name = unique_mapping_name("small");
        let size = 1;

        let h = create_named_mapping(&name, size).expect("create should succeed for small size");
        let hh = owned_handle_as_handle(&h);

        let p = unsafe { map_view(hh, size) }?;
        unsafe {
            let s = std::slice::from_raw_parts_mut(p, size);
            s[0] = 0x5A;
            assert_eq!(s[0], 0x5A);
            unmap_view(p);
        }

        Ok(())
    }
}
