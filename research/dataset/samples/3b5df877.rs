use windows::{
    core::{Result, PWSTR},
    Win32::UI::{
        Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
        WindowsAndMessaging::{SystemParametersInfoW, SPI_GETHIGHCONTRAST},
    },
};

fn is_high_contrast() -> Result<bool> {
    let mut hc = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        dwFlags: Default::default(),
        lpszDefaultScheme: PWSTR::null(),
    };

    // SAFETY: Calling SystemParametersInfoW with valid parameters and buffer
    unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            std::mem::size_of_val(&hc) as u32,
            Some(&mut hc as *mut _ as *mut _),
            Default::default(),
        )?;
    }

    Ok((HCF_HIGHCONTRASTON.0 & hc.dwFlags.0) != 0)
}

fn main() -> Result<()> {
    let high_contrast = is_high_contrast()?;
    println!(
        "High contrast mode is {}",
        if high_contrast { "enabled" } else { "disabled" }
    );
    Ok(())
}
