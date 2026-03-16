#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_in_conpty_echo_hello() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo hello"#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(output.contains("hello"), "Output should contain 'hello'");
        Ok(())
    }

    #[test]
    fn test_run_in_conpty_echo_world() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo world"#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(output.contains("world"), "Output should contain 'world'");
        Ok(())
    }

    #[test]
    fn test_run_in_conpty_empty_command() {
        let result = run_in_conpty("", 1000);
        assert!(result.is_err(), "Empty command should return an error");
    }

    #[test]
    fn test_run_in_conpty_nonexistent_command() {
        let result = run_in_conpty("nonexistent_command.exe", 1000);
        assert!(
            result.is_err(),
            "Nonexistent command should return an error"
        );
    }

    #[test]
    fn test_run_in_conpty_timeout() {
        let result = run_in_conpty(r#"cmd.exe /c timeout /t 5 /nobreak >nul & echo done"#, 100);
        assert!(
            result.is_err(),
            "Command exceeding timeout should return an error"
        );
    }

    #[test]
    fn test_run_in_conpty_multiple_lines() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo line1 && echo line2 && echo line3"#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(output.contains("line1"), "Output should contain 'line1'");
        assert!(output.contains("line2"), "Output should contain 'line2'");
        assert!(output.contains("line3"), "Output should contain 'line3'");
        Ok(())
    }

    #[test]
    fn test_run_in_conpty_command_with_spaces_in_argument() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo "hello world""#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(
            output.contains("hello world"),
            "Output should contain 'hello world'"
        );
        Ok(())
    }

    #[test]
    fn test_run_in_conpty_command_with_pipe() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo hello | findstr hello"#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(output.contains("hello"), "Output should contain 'hello'");
        Ok(())
    }

    #[test]
    fn test_run_in_conpty_stderr_capture() -> Result<(), std::io::Error> {
        let result = run_in_conpty(r#"cmd.exe /c echo error message >&2"#, 2000)?;
        assert!(!result.is_empty(), "Output should not be empty");
        let output = String::from_utf8_lossy(&result);
        assert!(
            output.contains("error message"),
            "Output should contain 'error message'"
        );
        Ok(())
    }
}