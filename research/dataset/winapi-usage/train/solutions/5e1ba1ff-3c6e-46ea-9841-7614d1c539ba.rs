#![deny(warnings)]

use windows::core::Result;
use windows::Win32::UI::Shell::{AssocCreate, CLSID_QueryAssociations, IQueryAssociations};

#[allow(dead_code)]
fn call_assoc_create() -> Result<IQueryAssociations> {
    // SAFETY: AssocCreate is a standard COM factory function that correctly initializes the object.
    unsafe { AssocCreate::<IQueryAssociations>(CLSID_QueryAssociations) }
}
