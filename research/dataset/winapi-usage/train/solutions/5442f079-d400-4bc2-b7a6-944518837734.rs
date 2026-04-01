use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Variant::InitVariantFromBuffer;

fn call_init_variant_from_buffer() -> windows::core::HRESULT {
    let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

    unsafe {
        match InitVariantFromBuffer(data.as_ptr() as *const std::ffi::c_void, data.len() as u32) {
            Ok(_) => S_OK,
            Err(hr) => hr.into(),
        }
    }
}
