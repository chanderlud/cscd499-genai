use windows::core::{Result, HSTRING};
use windows::Win32::System::Environment::SetEnvironmentVariableW;

pub fn set_process_env_var(name: &str, value: &str) -> Result<()> {
    let name = HSTRING::from(name);
    let value = HSTRING::from(value);

    unsafe { SetEnvironmentVariableW(&name, &value) }
}
