use windows::core::Result;
use windows::Win32::System::Com::ITypeInfo;
use windows::Win32::System::Ole::{CreateDispTypeInfo, INTERFACEDATA};

fn call_create_disp_type_info() -> Result<Result<()>> {
    let mut data = INTERFACEDATA {
        pmethdata: std::ptr::null_mut(),
        cMembers: 0,
    };
    let mut type_info: Option<ITypeInfo> = None;
    // SAFETY: We pass valid mutable pointers to local variables. The API expects these pointers to be valid for writing.
    unsafe { CreateDispTypeInfo(&mut data, 0, &mut type_info)? };
    Ok(Ok(()))
}
