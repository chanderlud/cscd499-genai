use windows::core::{Error, Result, HRESULT};
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreFromParsingName, GETPROPERTYSTOREFLAGS,
};

fn call_sh_get_property_store_from_parsing_name() -> HRESULT {
    let path = windows::core::w!("C:\\");
    unsafe {
        SHGetPropertyStoreFromParsingName::<_, _, IPropertyStore>(
            path,
            None,
            GETPROPERTYSTOREFLAGS(0),
        )
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
    }
}
