use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Search::SQLAllocConnect;

fn call_sql_alloc_connect() -> HRESULT {
    let ret = unsafe { SQLAllocConnect(std::ptr::null_mut(), std::ptr::null_mut()) };
    HRESULT(ret as i32)
}
