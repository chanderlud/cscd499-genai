use windows::core::Result;
use windows::Win32::System::Com::IBindCtx;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreFromParsingName, GETPROPERTYSTOREFLAGS,
};

fn call_sh_get_property_store_from_parsing_name() -> Result<IPropertyStore> {
    let path = windows::core::w!("C:\\Windows\\explorer.exe");
    // SAFETY: `path` is a valid null-terminated wide string literal.
    // `None` is a valid value for the optional `pbc` parameter.
    let store = unsafe {
        SHGetPropertyStoreFromParsingName::<_, _, IPropertyStore>(
            path,
            None::<&IBindCtx>,
            GETPROPERTYSTOREFLAGS(0),
        )
    }?;
    Ok(store)
}
