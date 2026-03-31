use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreFromParsingName, GETPROPERTYSTOREFLAGS,
};

fn call_sh_get_property_store_from_parsing_name() -> WIN32_ERROR {
    let result: Result<IPropertyStore> = unsafe {
        SHGetPropertyStoreFromParsingName(windows::core::w!("C:\\"), None, GETPROPERTYSTOREFLAGS(0))
    };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
