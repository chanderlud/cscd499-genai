use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::Storage::Packaging::Appx::{
    AddPackageDependency, AddPackageDependencyOptions, PACKAGEDEPENDENCY_CONTEXT,
};

fn call_add_package_dependency() -> HRESULT {
    let result = unsafe {
        AddPackageDependency(
            w!("test"),
            0,
            AddPackageDependencyOptions(0),
            std::ptr::null_mut::<PACKAGEDEPENDENCY_CONTEXT>(),
            None,
        )
    };
    result.map_or_else(|e| e.code(), |_| HRESULT(0))
}
