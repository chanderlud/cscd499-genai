use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Shell::{AssocCreate, CLSID_QueryAssociations, IQueryAssociations};

fn call_assoc_create() -> WIN32_ERROR {
    match unsafe { AssocCreate::<IQueryAssociations>(CLSID_QueryAssociations) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_else(|| WIN32_ERROR(e.code().0 as u32)),
    }
}
