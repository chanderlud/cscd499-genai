use windows::core::Result;
use windows::Win32::System::Com::{BindMoniker, IMoniker};

fn call_bind_moniker() -> Result<IMoniker> {
    // SAFETY: BindMoniker is an unsafe Win32 API. We pass None for the moniker and 0 for options.
    // The call may fail with an error code, which is correctly propagated as a Result.
    unsafe { BindMoniker::<_, IMoniker>(None::<&IMoniker>, 0) }
}
