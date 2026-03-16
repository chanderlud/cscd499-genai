// TITLE: Create an icon from RGBA pixel data using Win32 API

use std::mem;
use windows::core::Result;
use windows::Win32::UI::WindowsAndMessaging::{CreateIcon, DestroyIcon, HICON};

const PIXEL_SIZE: usize = 4;

#[repr(C)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    fn convert_to_bgra(&mut self) {
        mem::swap(&mut self.r, &mut self.b);
    }
}

fn create_icon_from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<HICON> {
    let pixel_count = rgba.len() / PIXEL_SIZE;
    let mut and_mask = Vec::with_capacity(pixel_count);

    // Convert RGBA to BGRA and create AND mask (invert alpha)
    let mut pixels: Vec<Pixel> = rgba
        .chunks_exact(PIXEL_SIZE)
        .map(|chunk| Pixel {
            r: chunk[0],
            g: chunk[1],
            b: chunk[2],
            a: chunk[3],
        })
        .collect();

    for pixel in &mut pixels {
        and_mask.push(pixel.a.wrapping_sub(u8::MAX)); // invert alpha channel
        pixel.convert_to_bgra();
    }

    // SAFETY: We're passing valid pointers to CreateIcon with correct dimensions
    let handle = unsafe {
        CreateIcon(
            None,
            width as i32,
            height as i32,
            1,
            (PIXEL_SIZE * 8) as u8,
            and_mask.as_ptr(),
            pixels.as_ptr() as *const u8,
        )
    }?;

    Ok(handle)
}

fn main() -> Result<()> {
    // Create a simple 2x2 red icon with varying alpha
    let rgba_data = vec![
        255, 0, 0, 255, // Red, fully opaque
        255, 0, 0, 128, // Red, half transparent
        255, 0, 0, 64, // Red, mostly transparent
        255, 0, 0, 0, // Red, fully transparent
    ];

    let icon = create_icon_from_rgba(rgba_data, 2, 2)?;

    // SAFETY: We're destroying a valid icon handle that we created
    unsafe {
        DestroyIcon(icon)?;
    }

    println!("Icon created and destroyed successfully");
    Ok(())
}
