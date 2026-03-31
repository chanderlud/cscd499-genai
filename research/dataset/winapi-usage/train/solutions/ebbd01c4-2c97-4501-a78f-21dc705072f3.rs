use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;

fn call_expand_environment_strings_w() -> WIN32_ERROR {
    let src = w!("%TEMP%");
    let mut dst = [0u16; 260];
    // SAFETY: src is a valid PCWSTR, and dst is a valid mutable buffer.
    let ret = unsafe { ExpandEnvironmentStringsW(src, Some(&mut dst)) };
    WIN32_ERROR(ret)
}
