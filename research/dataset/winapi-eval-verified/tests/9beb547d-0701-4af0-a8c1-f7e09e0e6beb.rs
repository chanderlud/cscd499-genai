#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{
        env,
        fs::{self, OpenOptions},
        io::Write,
        mem::size_of,
        path::{Path, PathBuf},
        slice,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use windows::Win32::System::Memory::{UnmapViewOfFile, VirtualQuery, MEMORY_BASIC_INFORMATION, MEMORY_MAPPED_VIEW_ADDRESS, MEM_MAPPED, PAGE_GUARD, PAGE_NOCACHE, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOMBINE, PAGE_WRITECOPY};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let mut p = env::temp_dir();
        let pid = std::process::id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        p.push(format!("{prefix}_{pid}_{now}_{n}"));
        p
    }

    fn write_new_file(path: &Path, bytes: &[u8]) {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        if !bytes.is_empty() {
            f.write_all(bytes).unwrap();
        }
        f.flush().unwrap();
    }

    /// Best-effort cleanup if something panics mid-test.
    struct UnmapOnDrop(*const u8);
    impl Drop for UnmapOnDrop {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: self.0 as _ });
                }
            }
        }
    }

    fn assert_view_is_mapped_readonly(ptr: *const u8) {
        assert!(!ptr.is_null(), "mapped pointer must not be null for non-empty files");

        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        let got = unsafe { VirtualQuery(Some(ptr as _), &mut mbi, size_of::<MEMORY_BASIC_INFORMATION>()) };
        assert!(
            got >= size_of::<MEMORY_BASIC_INFORMATION>(),
            "VirtualQuery returned too little: {got}"
        );

        assert_eq!(
            mbi.Type, MEM_MAPPED,
            "expected MEM_MAPPED view, got {:?}",
            mbi.Type
        );

        // Protection can include extra bits like GUARD/NOCACHE/WRITECOMBINE.
        let masked = mbi.Protect.0 & !(PAGE_GUARD.0 | PAGE_NOCACHE.0 | PAGE_WRITECOMBINE.0);

        // Read-only mapping should not come back as PAGE_READWRITE.
        assert_ne!(
            masked, PAGE_READWRITE.0,
            "mapping should not be PAGE_READWRITE"
        );

        // Common acceptable values for a read-only view.
        assert!(
            masked == PAGE_READONLY.0 || masked == PAGE_WRITECOPY.0,
            "unexpected protection flags for read-only map: 0x{masked:08x}"
        );
    }

    #[test]
    fn map_ro_maps_entire_file_and_matches_contents() {
        let path = unique_temp_path("map_ro_basic");

        // Weird size (not page aligned) to catch lazy implementations.
        let mut bytes = vec![0u8; 8192 + 123];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i as u32 * 31).wrapping_add(7) as u8;
        }
        write_new_file(&path, &bytes);

        let (fh, mh, v) = map_ro(&path).expect("map_ro should succeed on a normal file");

        assert_eq!(
            v.len,
            bytes.len(),
            "len must come from file size (entire file)"
        );
        assert!(!v.ptr.is_null());

        // The view should stay readable even if the file handle is dropped (mapping handle kept).
        drop(fh);

        let guard = UnmapOnDrop(v.ptr);
        let view = unsafe { slice::from_raw_parts(v.ptr, v.len) };
        assert_eq!(view, bytes.as_slice(), "mapped bytes must match file bytes");

        assert_view_is_mapped_readonly(v.ptr);

        // Explicit unmap to avoid leaking address space into other tests.
        unsafe {
            assert!(
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: v.ptr as _ }).is_ok(),
                "UnmapViewOfFile failed"
            );
        }
        std::mem::forget(guard);

        drop(mh);
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn map_ro_fails_for_missing_file() {
        let path = unique_temp_path("map_ro_missing");
        let r = map_ro(&path);
        assert!(r.is_err(), "expected error for missing file");
    }

    #[test]
    fn map_ro_fails_for_directory_path() {
        let path = unique_temp_path("map_ro_dir");
        fs::create_dir(&path).unwrap();

        let r = map_ro(&path);
        assert!(r.is_err(), "expected error when mapping a directory");

        fs::remove_dir(&path).unwrap();
    }

    #[test]
    fn map_ro_empty_file_returns_error() {
        let path = unique_temp_path("map_ro_empty");
        write_new_file(&path, &[]);

        let r = map_ro(&path);
        assert!(
            r.is_err(),
            "empty files cannot be meaningfully mapped as a view"
        );

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn map_ro_allows_cleanup_after_unmap_and_drop_handles() {
        let path = unique_temp_path("map_ro_cleanup");
        let bytes = b"cleanup-check-bytes";
        write_new_file(&path, bytes);

        let (_fh, mh, v) = map_ro(&path).expect("map_ro should succeed");
        let guard = UnmapOnDrop(v.ptr);

        // Touch the mapping so this isn't a “it returned a pointer, trust me bro” test.
        let view = unsafe { slice::from_raw_parts(v.ptr, v.len) };
        assert_eq!(view, bytes);

        unsafe {
            assert!(
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: v.ptr as _ }).is_ok(),
                "UnmapViewOfFile failed"
            );
        }
        std::mem::forget(guard);
        drop(mh);

        // If the view wasn’t unmapped or handles are held incorrectly, this can fail on Windows.
        fs::remove_file(&path).unwrap();
    }
}
