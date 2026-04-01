use windows::Win32::System::Environment::CreateEnvironmentBlock;

fn call_create_environment_block() -> windows::core::Result<windows::core::Result<()>> {
    // Create a null pointer to receive the environment block
    let mut env_block: *mut core::ffi::c_void = std::ptr::null_mut();

    // Call CreateEnvironmentBlock with concrete parameters:
    // - env_block: pointer to receive the environment block
    // - None: use current process token
    // - true: inherit environment variables
    let inner_result: windows::core::Result<()> =
        unsafe { CreateEnvironmentBlock(&mut env_block, None, true) };

    // Wrap the inner Result in an outer Result
    Ok(inner_result)
}
