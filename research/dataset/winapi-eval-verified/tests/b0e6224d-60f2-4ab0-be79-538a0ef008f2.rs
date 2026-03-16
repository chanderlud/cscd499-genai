#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_nanos();

            let path = std::env::temp_dir().join(format!(
                "run_in_job_with_timeout_{name}_{}_{}",
                std::process::id(),
                unique
            ));

            fs::create_dir_all(&path).expect("failed to create temporary test directory");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn powershell_file_command(script: &Path) -> String {
        format!(
            r#"powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{}""#,
            script.display()
        )
    }

    fn powershell_literal_path(path: &Path) -> String {
        path.display().to_string().replace('\'', "''")
    }

    fn write_process_tree_scripts(root: &Path) -> (PathBuf, PathBuf) {
        let parent = root.join("parent.ps1");
        let child = root.join("child.ps1");
        let marker = root.join("child-ran.txt");

        fs::write(
            &child,
            format!(
                r#"
Start-Sleep -Milliseconds 750
Set-Content -LiteralPath '{}' -Value 'child-ran'
"#,
                powershell_literal_path(&marker)
            )
            .trim_start(),
        )
        .expect("failed to write child script");

        fs::write(
            &parent,
            format!(
                r#"
Start-Process -WindowStyle Hidden -FilePath 'powershell.exe' -ArgumentList @(
    '-NoProfile',
    '-NonInteractive',
    '-ExecutionPolicy', 'Bypass',
    '-File', '{}'
)
Start-Sleep -Seconds 3
"#,
                powershell_literal_path(&child)
            )
            .trim_start(),
        )
        .expect("failed to write parent script");

        (parent, marker)
    }

    #[test]
    fn returns_false_when_process_finishes_before_timeout() {
        let command = r#"powershell.exe -NoProfile -NonInteractive -Command "exit 0""#;

        let killed = run_in_job_with_timeout(command, 2_000)
            .expect("run_in_job_with_timeout should succeed for a quick command");

        assert!(
            !killed,
            "a process that exits immediately should not be reported as killed by the job"
        );
    }

    #[test]
    fn returns_true_when_process_exceeds_timeout() {
        let command =
            r#"powershell.exe -NoProfile -NonInteractive -Command "Start-Sleep -Seconds 5""#;

        let killed = run_in_job_with_timeout(command, 200)
            .expect("run_in_job_with_timeout should succeed for a slow command");

        assert!(
            killed,
            "a process that outlives the timeout should be reported as killed by the job"
        );
    }

    #[test]
    fn allows_child_process_to_finish_when_timeout_is_long_enough() {
        let dir = TestDir::new("child_completes");
        let (parent, marker) = write_process_tree_scripts(dir.path());

        let killed = run_in_job_with_timeout(&powershell_file_command(&parent), 5_000)
            .expect("run_in_job_with_timeout should succeed for the parent/child script");

        assert!(
            !killed,
            "the job should not time out when given a long enough deadline"
        );
        assert!(
            marker.exists(),
            "the child marker file should exist when the child is allowed to finish"
        );
    }

    #[test]
    fn timeout_kills_the_entire_process_tree() {
        let dir = TestDir::new("tree_killed");
        let (parent, marker) = write_process_tree_scripts(dir.path());

        let killed = run_in_job_with_timeout(&powershell_file_command(&parent), 200)
            .expect("run_in_job_with_timeout should succeed for the parent/child script");

        assert!(killed, "the parent process should time out");

        thread::sleep(Duration::from_millis(1_500));

        assert!(
            !marker.exists(),
            "the child marker file should never be created if the whole process tree is killed"
        );
    }
}
