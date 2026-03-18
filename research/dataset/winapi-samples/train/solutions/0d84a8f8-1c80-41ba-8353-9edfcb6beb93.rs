use std::path::Path;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Security::Cryptography::{
    BCryptCloseAlgorithmProvider, BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash,
    BCryptHashData, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE,
    BCRYPT_HASH_REUSABLE_FLAG,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_FLAG_SEQUENTIAL_SCAN, FILE_GENERIC_READ, FILE_SHARE_READ,
    OPEN_EXISTING,
};

const BUFFER_SIZE: usize = 4096;

struct Algorithm {
    handle: BCRYPT_ALG_HANDLE,
}

impl Algorithm {
    fn new(algorithm_id: &str) -> Result<Self> {
        let wide_id = wide_null(std::ffi::OsStr::new(algorithm_id));
        let mut handle = BCRYPT_ALG_HANDLE::default();
        unsafe {
            BCryptOpenAlgorithmProvider(
                &mut handle,
                PCWSTR::from_raw(wide_id.as_ptr()),
                None,
                BCRYPT_HASH_REUSABLE_FLAG,
            )
            .ok()?;
        }
        Ok(Self { handle })
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.handle, 0);
        }
    }
}

struct Hash {
    handle: BCRYPT_HASH_HANDLE,
}

impl Hash {
    fn new(algorithm: &Algorithm) -> Result<Self> {
        let mut handle = BCRYPT_HASH_HANDLE::default();
        unsafe {
            BCryptCreateHash(algorithm.handle, &mut handle, None, None, 0).ok()?;
        }
        Ok(Self { handle })
    }

    fn update(&mut self, data: &[u8]) -> Result<()> {
        unsafe {
            BCryptHashData(self.handle, data, 0).ok()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<Vec<u8>> {
        let mut hash_value = vec![0u8; 32]; // SHA-256 produces 32 bytes
        unsafe {
            BCryptFinishHash(self.handle, &mut hash_value, 0).ok()?;
        }
        Ok(hash_value)
    }
}

impl Drop for Hash {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.handle);
        }
    }
}

struct FileHandle(HANDLE);

impl FileHandle {
    fn open(path: &Path) -> Result<Self> {
        let wide_path = wide_null(path.as_os_str());
        let handle = unsafe {
            CreateFileW(
                PCWSTR::from_raw(wide_path.as_ptr()),
                FILE_GENERIC_READ.0,
                FILE_SHARE_READ,
                None,
                OPEN_EXISTING,
                FILE_FLAG_SEQUENTIAL_SCAN,
                None,
            )?
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(Error::from_thread());
        }
        Ok(Self(handle))
    }

    fn read_chunk(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut bytes_read = 0u32;
        unsafe {
            ReadFile(
                self.0,
                Some(buffer),
                Some(&mut bytes_read as *mut u32),
                None,
            )?;
        }
        Ok(bytes_read as usize)
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

fn verify_file_hash(file_path: &Path, expected_hash: &[u8], algorithm: &Algorithm) -> Result<bool> {
    let file = FileHandle::open(file_path)?;
    let mut hash = Hash::new(algorithm)?;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = file.read_chunk(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hash.update(&buffer[..bytes_read])?;
    }

    let computed_hash = hash.finish()?;
    Ok(computed_hash == expected_hash)
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}
