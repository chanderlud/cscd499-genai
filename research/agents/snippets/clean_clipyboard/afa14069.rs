use std::char::decode_utf16;

use windows::core::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardFormatNameW, OpenClipboard,
};

fn main() -> Result<(), Error> {
    unsafe { OpenClipboard(Some(HWND(std::ptr::null_mut()))) }?;

    println!("Available clipboard formats:");
    println!("-------------------------");

    let mut format: u32 = 0;
    loop {
        format = unsafe { EnumClipboardFormats(format) };
        if format == 0 {
            break;
        }

        let mut name_buffer = [0u16; 256];
        let name_length = unsafe { GetClipboardFormatNameW(format, &mut name_buffer) };

        if name_length > 0 {
            let name = decode_utf16(name_buffer[..name_length as usize].iter().copied())
                .map(|r| r.unwrap_or(std::char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            println!("Format ID: {:?}, Name: {}", format, name);
        } else {
            println!("Format ID: {:?}, Name: (unknown)", format);
        }
    }

    unsafe { CloseClipboard() }?;
    Ok(())
}
