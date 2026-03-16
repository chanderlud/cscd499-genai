use windows::core::{Result, Error};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

pub fn align_mapping_offset(offset: u64) -> Result<(u64, u64, u32)> {
    let mut system_info = SYSTEM_INFO::default();
    
    // SAFETY: GetSystemInfo writes to a SYSTEM_INFO struct, which is valid for this call
    unsafe {
        GetSystemInfo(&mut system_info);
    }
    
    let granularity = system_info.dwAllocationGranularity;
    let granularity_u64 = granularity as u64;
    
    // Calculate aligned base by rounding down to nearest multiple of granularity
    let aligned_base = offset - (offset % granularity_u64);
    let delta = offset - aligned_base;
    
    Ok((aligned_base, delta, granularity))
}