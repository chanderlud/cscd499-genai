// TITLE: Checking High Contrast Mode with SystemParametersInfoA (ANSI version)

use windows::{
    core::{Result, PSTR},
    Win32::UI::Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTA},
    Win32::UI::WindowsAndMessaging::{SystemParametersInfoA, SPI_GETHIGHCONTRAST},
};

fn is_high_contrast() -> Result<bool> {
    let mut hc = HIGHCONTRASTA {
        cbSize: std::mem::size_of::<HIGHCONTRASTA>() as u32,
        dwFlags: Default::default(),
        lpszDefaultScheme: PSTR::null(),
    };

    // SAFETY: We're calling a system API with properly initialized struct
    unsafe {
        SystemParametersInfoA(
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
        if high_contrast { "ON" } else { "OFF" }
    );
    Ok(())
}
