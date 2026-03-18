use std::path::{Path, PathBuf, Component};
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn create_single_dir(path: &Path) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    unsafe {
        match CreateDirectoryW(PCWSTR(wide_path.as_ptr()), None) {
            Ok(_) => Ok(()),
            Err(_) => {
                let err = GetLastError();
                if err == ERROR_ALREADY_EXISTS {
                    Ok(())
                } else {
                    Err(Error::from_hresult(windows::core::HRESULT::from_win32(err.0)))
                }
            }
        }
    }
}

pub fn create_dir_all(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();

    for component in path.components() {
        current.push(component);

        match component {
            Component::Prefix(_) | Component::RootDir => continue,
            Component::Normal(_) => {
                create_single_dir(&current)?;
            }
            Component::CurDir | Component::ParentDir => continue,
        }
    }

    Ok(())
}