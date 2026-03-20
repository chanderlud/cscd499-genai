use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        System::Threading::{
            GetCurrentThread, GetCurrentThreadId, GetThreadDescription, SetThreadDescription,
        },
    },
    core::{HSTRING, PWSTR, Result},
};

fn take_local_pwstr(raw: PWSTR) -> String {
    unsafe {
        if raw.is_null() {
            return String::new();
        }

        let text = raw
            .to_string()
            .expect("GetThreadDescription returned invalid UTF-16");
        let _ = LocalFree(Some(HLOCAL(raw.0.cast())));
        text
    }
}

pub fn label_current_thread(tag: &str) -> Result<String> {
    let thread = unsafe { GetCurrentThread() };
    let thread_id = unsafe { GetCurrentThreadId() };
    let description = format!("tid-{thread_id}-{tag}");
    let description_h = HSTRING::from(description);

    unsafe {
        SetThreadDescription(thread, &description_h)?;
        let raw = GetThreadDescription(thread)?;
        Ok(take_local_pwstr(raw))
    }
}
