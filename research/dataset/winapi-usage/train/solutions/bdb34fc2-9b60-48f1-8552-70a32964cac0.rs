use windows::core::HRESULT;
use windows::Win32::UI::Shell::{AssocCreate, CLSID_QueryAssociations, IQueryAssociations};

fn call_assoc_create() -> HRESULT {
    unsafe {
        AssocCreate::<IQueryAssociations>(CLSID_QueryAssociations)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
