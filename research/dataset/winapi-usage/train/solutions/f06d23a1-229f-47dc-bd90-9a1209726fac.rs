use windows::core::{Error, Result};
use windows::Win32::System::Ole::{CreateOleAdviseHolder, IOleAdviseHolder};

fn call_create_ole_advise_holder() -> Result<Result<IOleAdviseHolder>> {
    let result = unsafe { CreateOleAdviseHolder() };
    Ok(result)
}
