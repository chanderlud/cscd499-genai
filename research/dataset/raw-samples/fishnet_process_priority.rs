fn set_current_process_min_priority() -> windows::core::Result<()> {
    use windows::Win32::System::Threading::{
        BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, SetPriorityClass,
    };

    // BELOW_NORMAL_PRIORITY_CLASS is the lowest priority that won't completely
    // starve tasks of CPU time on high loads. The lowest IDLE_PRIORITY_CLASS
    // is stricter than Linux's nice 19!
    unsafe { SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS) }
}