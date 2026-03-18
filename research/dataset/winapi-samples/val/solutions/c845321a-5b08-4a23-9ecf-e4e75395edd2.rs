use windows::core::*;
use windows::Win32::System::Com::{CoInitialize, CoUninitialize, CreateUri, URI_CREATE_FLAGS};

fn main() -> windows::core::Result<()> {
    unsafe {
        CoInitialize(None).ok()?;
        let uri = CreateUri(w!("http://kennykerr.ca"), URI_CREATE_FLAGS::default(), None)?;
        let domain = uri.GetDomain().unwrap_or_default();
        let port = uri.GetPort().unwrap_or_default();
        println!("{:?} ({port})", domain);
        CoUninitialize();
    }
    Ok(())
}
