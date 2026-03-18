use std::ptr;
use windows::Win32::Foundation::{E_NOINTERFACE, E_POINTER};
use windows::Win32::NetworkManagement::NetManagement::{
    NetApiBufferFree, NetUserEnum, NET_USER_ENUM_FILTER_FLAGS, USER_INFO_0,
};

fn enumerate_local_users() -> Result<Vec<String>, u32> {
    let mut buf_ptr: *mut u8 = std::ptr::null_mut();
    let mut entries_read: u32 = 0;
    let mut total_entries: u32 = 0;

    let status = unsafe {
        NetUserEnum(
            None,
            0,
            NET_USER_ENUM_FILTER_FLAGS(0),
            &mut buf_ptr as *mut _,
            u32::MAX,
            &mut entries_read,
            &mut total_entries,
            None,
        )
    };

    if status != 0 {
        return Err(status);
    }

    let mut users = Vec::new();
    if entries_read > 0 {
        let info_array = buf_ptr as *const USER_INFO_0;
        for i in 0..entries_read {
            let info = unsafe { &*info_array.add(i as usize) };
            let username = unsafe { info.usri0_name.to_string().unwrap_or_default() };
            users.push(username);
        }
    }

    if buf_ptr != std::ptr::null_mut() {
        unsafe {
            NetApiBufferFree(Some(buf_ptr as *const core::ffi::c_void));
        }
    }

    Ok(users)
}
