#[cfg(all(test, windows))]
mod tests {
    use super::*;

    use std::{
        env,
        fs::{self, OpenOptions},
        os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, OwnedHandle},
        path::{Path, PathBuf},
        ptr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use windows::{
        core::PCWSTR,
        Win32::Foundation::{CloseHandle, HANDLE},
        Win32::System::Memory::{
            CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_WRITE, PAGE_READWRITE,
        },
    };
    use windows::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS;

    const FILE_LEN: usize = 64 * 1024;

    fn unique_temp_path(tag: &str) -> PathBuf {
        let mut p = env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("flush_mapped_{tag}_{}_{}.bin", std::process::id(), stamp));
        p
    }

    struct TempFile {
        path: PathBuf,
    }
    impl TempFile {
        fn new(tag: &str) -> Self {
            Self { path: unique_temp_path(tag) }
        }
        fn path(&self) -> &Path {
            &self.path
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            // Best-effort cleanup. If something still has the file open, Windows will say no.
            let _ = fs::remove_file(&self.path);
        }
    }

    fn open_rw_owned(path: &Path, len: usize) -> OwnedHandle {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .expect("create temp file");
        f.set_len(len as u64).expect("set_len");
        let raw = f.into_raw_handle();
        unsafe { OwnedHandle::from_raw_handle(raw) }
    }

    fn open_ro_owned(path: &Path) -> OwnedHandle {
        let f = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .expect("open read-only");
        let raw = f.into_raw_handle();
        unsafe { OwnedHandle::from_raw_handle(raw) }
    }

    struct MappedView {
        ptr: *mut u8,
        len: usize,
        mapping: HANDLE,
    }
    impl MappedView {
        fn new(file: &OwnedHandle, len: usize) -> Self {
            unsafe {
                let hfile = HANDLE(file.as_raw_handle() as _);

                let mapping = CreateFileMappingW(
                    hfile,
                    None,
                    PAGE_READWRITE,
                    0,
                    len as u32,
                    PCWSTR::null(),
                ).unwrap();

                let p = MapViewOfFile(mapping, FILE_MAP_WRITE, 0, 0, len);
                assert!(!p.Value.is_null(), "MapViewOfFile failed");

                Self {
                    ptr: p.Value as *mut u8,
                    len,
                    mapping,
                }
            }
        }

        fn write_at(&mut self, offset: usize, bytes: &[u8]) {
            assert!(offset + bytes.len() <= self.len);
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.add(offset), bytes.len());
            }
        }
    }
    impl Drop for MappedView {
        fn drop(&mut self) {
            unsafe {
                if !self.ptr.is_null() {
                    let _ = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS { Value: self.ptr as *mut _ });
                }
                if !self.mapping.is_invalid() {
                    let _ = CloseHandle(self.mapping);
                }
            }
        }
    }

    #[test]
    fn flush_persists_mapped_changes() {
        let tmp = TempFile::new("persists");

        let a = b"hello from a mapped view";
        let b = b"and also near the end";

        // Scope ensures handles/views are dropped before reading back.
        {
            let fh = open_rw_owned(tmp.path(), FILE_LEN);
            let mut v = MappedView::new(&fh, FILE_LEN);

            v.write_at(0, a);
            v.write_at(FILE_LEN - b.len(), b);

            flush_mapped(v.ptr as *const u8, v.len, &fh).expect("flush_mapped should succeed");
        }

        let data = fs::read(tmp.path()).expect("read back");
        assert_eq!(&data[..a.len()], a);
        assert_eq!(&data[FILE_LEN - b.len()..], b);
    }

    #[test]
    fn flush_len_zero_flushes_entire_view() {
        let tmp = TempFile::new("len0");

        {
            let fh = open_rw_owned(tmp.path(), FILE_LEN);
            let mut v = MappedView::new(&fh, FILE_LEN);

            let msg = b"len==0 should still flush";
            v.write_at(1234, msg);

            // Per Win32 semantics, 0 bytes means “flush the whole view”.
            flush_mapped(v.ptr as *const u8, 0, &fh).expect("flush_mapped(len=0) should succeed");
        }

        let data = fs::read(tmp.path()).expect("read back");
        assert_eq!(&data[1234..1234 + b"len==0 should still flush".len()], b"len==0 should still flush");
    }

    #[test]
    fn flush_errors_on_null_view_pointer() {
        let tmp = TempFile::new("null_view");
        let fh = open_rw_owned(tmp.path(), FILE_LEN);

        // If you’re not calling FlushViewOfFile, you might accidentally “succeed” here.
        assert!(
            flush_mapped(ptr::null(), 0, &fh).is_err(),
            "null view pointer must error"
        );
    }

    #[test]
    fn flush_errors_with_read_only_file_handle() {
        let tmp = TempFile::new("ro_handle");

        // Create file + mapping using a write handle...
        let rw = open_rw_owned(tmp.path(), FILE_LEN);
        let mut v = MappedView::new(&rw, FILE_LEN);
        v.write_at(0, b"x");

        // ...but attempt the durability step using a read-only handle.
        // This should fail specifically because FlushFileBuffers needs write access.
        let ro = open_ro_owned(tmp.path());
        assert!(
            flush_mapped(v.ptr as *const u8, v.len, &ro).is_err(),
            "read-only handle must error (verifies FlushFileBuffers is called)"
        );
    }

    #[test]
    fn flush_errors_when_len_exceeds_view() {
        let tmp = TempFile::new("len_too_big");
        let fh = open_rw_owned(tmp.path(), FILE_LEN);
        let v = MappedView::new(&fh, FILE_LEN);

        assert!(
            flush_mapped(v.ptr as *const u8, v.len + 1, &fh).is_err(),
            "len beyond mapped view must error"
        );
    }
}