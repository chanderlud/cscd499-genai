#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::{
        Foundation::{HLOCAL, LocalFree},
        System::Threading::{GetCurrentThread, GetCurrentThreadId, GetThreadDescription},
    };

    fn read_current_thread_description() -> String {
        unsafe {
            let raw = GetThreadDescription(GetCurrentThread())
                .expect("GetThreadDescription should succeed in test");
            let text = raw
                .to_string()
                .expect("thread description should be valid UTF-16");
            let _ = LocalFree(Some(HLOCAL(raw.0.cast())));
            text
        }
    }

    #[test]
    fn sets_expected_description() {
        let expected = format!("tid-{}-worker", unsafe { GetCurrentThreadId() });
        let actual = label_current_thread("worker").expect("label_current_thread should succeed");

        assert_eq!(actual, expected);
        assert_eq!(read_current_thread_description(), expected);
    }

    #[test]
    fn overwrites_previous_description() {
        label_current_thread("first").expect("initial description should succeed");
        let second =
            label_current_thread("second").expect("second description update should succeed");

        assert!(second.ends_with("-second"));
        assert_eq!(read_current_thread_description(), second);
        assert!(!second.ends_with("-first"));
    }
}