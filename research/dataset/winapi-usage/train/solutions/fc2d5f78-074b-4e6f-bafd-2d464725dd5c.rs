#![allow(unused)]
use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::JobObjects::CreateJobObjectW;

fn call_create_job_object_w() -> HRESULT {
    match unsafe { CreateJobObjectW(None, windows::core::w!("MyJobObject")) } {
        Ok(_) => HRESULT(0),
        Err(e) => e.code(),
    }
}
