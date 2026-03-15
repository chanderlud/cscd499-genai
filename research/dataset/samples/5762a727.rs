use windows::{
    core::{w, Result},
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    },
};

fn read_apps_use_light_theme() -> Result<Option<bool>> {
    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;

    // SAFETY: Calling RegGetValueW with valid parameters
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut _ as _),
            Some(&mut data_size),
        )
    };

    if status == ERROR_SUCCESS {
        Ok(Some(data != 0))
    } else {
        // Convert WIN32_ERROR to HRESULT and then to Error
        Err(windows::core::Error::from_hresult(
            windows::core::HRESULT::from_win32(status.0),
        ))
    }
}

fn main() -> Result<()> {
    match read_apps_use_light_theme()? {
        Some(true) => println!("System uses light theme for apps"),
        Some(false) => println!("System uses dark theme for apps"),
        None => println!("Could not read theme setting"),
    }
    Ok(())
}
