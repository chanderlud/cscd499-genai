use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::Networking::WinInet::AppCacheCreateAndCommitFile;

fn call_app_cache_create_and_commit_file() -> Result<u32> {
    let result = unsafe {
        AppCacheCreateAndCommitFile(
            std::ptr::null(),
            w!("C:\\source.txt"),
            w!("http://example.com/file.txt"),
            &[],
        )
    };

    if result == 0 {
        Ok(result)
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(result)))
    }
}
