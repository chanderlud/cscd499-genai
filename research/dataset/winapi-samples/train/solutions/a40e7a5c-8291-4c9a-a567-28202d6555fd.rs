use windows::core::Result;
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

pub fn align_mapping_offset_bitwise(offset: u64) -> Result<(u64, u64, u32)> {
    let mut system_info = SYSTEM_INFO::default();
    // SAFETY: GetSystemInfo writes to a valid SYSTEM_INFO struct
    unsafe {
        GetSystemInfo(&mut system_info);
    }

    let granularity = system_info.dwAllocationGranularity;
    let granularity_u64 = granularity as u64;

    // Since granularity is guaranteed to be a power of two,
    // we can compute the mask by subtracting 1 and bitwise NOT
    let mask = !(granularity_u64 - 1);
    let base = offset & mask;
    let delta = offset - base;

    Ok((base, delta, granularity))
}
