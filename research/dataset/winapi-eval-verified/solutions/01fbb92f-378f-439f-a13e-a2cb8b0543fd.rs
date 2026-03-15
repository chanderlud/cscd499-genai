use windows::Foundation::Collections::IPropertySet;
use windows::Foundation::{IPropertyValue, PropertyType};
use windows::Storage::{ApplicationData, ApplicationDataContainer};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::core::{Error, HSTRING, IInspectable, Interface, Result};

pub fn local_settings_set(key: &str, value: &str) -> Result<()> {
    let key_hstring = HSTRING::from(key);
    let value_hstring = HSTRING::from(value);

    // Get current ApplicationData
    let app_data = ApplicationData::Current()?;

    // Get LocalSettings container
    let local_settings: ApplicationDataContainer = app_data.LocalSettings()?;

    // Get Values property set
    let values: IPropertySet = local_settings.Values()?;

    // Create IInspectable from string value using PropertyValue
    let inspectable: IInspectable =
        windows::Foundation::PropertyValue::CreateString(&value_hstring)?;

    // Insert the key-value pair
    values.Insert(&key_hstring, &inspectable)?;

    Ok(())
}

pub fn local_settings_get(key: &str) -> Result<Option<String>> {
    let key_hstring = HSTRING::from(key);

    let app_data = ApplicationData::Current()?;
    let local_settings: ApplicationDataContainer = app_data.LocalSettings()?;
    let values: IPropertySet = local_settings.Values()?;

    if !values.HasKey(&key_hstring)? {
        return Ok(None);
    }

    let inspectable: IInspectable = values.Lookup(&key_hstring)?;

    let property_value: IPropertyValue = inspectable
        .cast()
        .map_err(|_| Error::new(E_INVALIDARG, "Value exists but is not a property value"))?;

    if property_value.Type()? != PropertyType::String {
        return Err(Error::new(E_INVALIDARG, "Value exists but is not a string"));
    }

    let hstring = property_value.GetString()?;
    Ok(Some(hstring.to_string()))
}
