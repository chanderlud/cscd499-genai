#[cfg(test)]
mod tests {
    use super::*;

    // Raw JOB_OBJECT_MSG_* values from winnt.h / jobapi2 docs.
    const JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO: u32 = 4;
    const JOB_OBJECT_MSG_NEW_PROCESS: u32 = 6;
    const JOB_OBJECT_MSG_EXIT_PROCESS: u32 = 7;
    const JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS: u32 = 8;

    fn assert_lifecycle_notifications(messages: &[u32]) {
        assert!(
            !messages.is_empty(),
            "expected at least one job notification, got none"
        );

        for &msg in messages {
            assert!(
                matches!(
                    msg,
                    JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO
                        | JOB_OBJECT_MSG_NEW_PROCESS
                        | JOB_OBJECT_MSG_EXIT_PROCESS
                        | JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS
                ),
                "unexpected job notification id {msg} in {messages:?}"
            );
        }

        assert!(
            messages.iter().any(|&msg| {
                matches!(
                    msg,
                    JOB_OBJECT_MSG_EXIT_PROCESS
                        | JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS
                        | JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO
                )
            }),
            "expected at least one terminal job notification in {messages:?}"
        );
    }

    #[test]
    fn returns_exit_code_and_job_notifications_for_simple_process() {
        let (exit_code, messages) =
            run_in_job_collect_messages(r#"cmd.exe /c exit 7"#, 2_000)
                .expect("run_in_job_collect_messages should succeed for a simple child");

        assert_eq!(exit_code, 7);
        assert_lifecycle_notifications(&messages);
    }

    #[test]
    fn returns_correct_results_across_repeated_calls() {
        for expected in [0_u32, 3, 7, 42] {
            let command = format!(r#"cmd.exe /c exit {expected}"#);
            let (exit_code, messages) = run_in_job_collect_messages(&command, 2_000)
                .unwrap_or_else(|e| panic!("command `{command}` failed: {e}"));

            assert_eq!(exit_code, expected, "wrong exit code for `{command}`");
            assert_lifecycle_notifications(&messages);
        }
    }
}