use windows::core::{Error, Result};
use windows::Win32::Storage::Packaging::Appx::{
    AddPackageDependency, AddPackageDependencyOptions, PACKAGEDEPENDENCY_CONTEXT,
};

fn call_add_package_dependency() -> Result<()> {
    let mut context = PACKAGEDEPENDENCY_CONTEXT(std::ptr::null_mut());
    unsafe {
        AddPackageDependency(
            windows::core::w!("test-dependency-id"),
            0,
            AddPackageDependencyOptions::default(),
            &mut context,
            None,
        )?;
    }
    Ok(())
}
