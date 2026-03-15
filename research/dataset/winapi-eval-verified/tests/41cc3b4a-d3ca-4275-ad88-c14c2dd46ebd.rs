#[cfg(windows)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_message() {
        let out = shm_event_handshake("ipc_shm_06", b"ping").unwrap();
        assert_eq!(out, b"ping");
    }

    #[test]
    fn test_empty_message() {
        let out = shm_event_handshake("ipc_shm_empty", b"").unwrap();
        assert_eq!(out, b"");
    }

    #[test]
    fn test_long_message() {
        // Message exactly at buffer size (4096) should work
        let msg = vec![0u8; 4096];
        let out = shm_event_handshake("ipc_shm_max", &msg).unwrap();
        assert_eq!(out.len(), 4096);
    }

    #[test]
    fn test_message_too_large() {
        // Message larger than buffer should error
        let msg = vec![0u8; 4097];
        let result = shm_event_handshake("ipc_shm_too_large", &msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_message() {
        let msg = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        let out = shm_event_handshake("ipc_shm_binary", &msg).unwrap();
        assert_eq!(out, msg);
    }

    #[test]
    fn test_ascii_message() {
        let msg = b"Hello, World! This is a test message.";
        let out = shm_event_handshake("ipc_shm_ascii", msg).unwrap();
        assert_eq!(out, msg);
    }

    #[test]
    fn test_multiple_messages_different_names() {
        // Each test uses different name to avoid conflicts
        let out1 = shm_event_handshake("ipc_shm_07a", b"first").unwrap();
        assert_eq!(out1, b"first");
        
        let out2 = shm_event_handshake("ipc_shm_07b", b"second").unwrap();
        assert_eq!(out2, b"second");
    }
}
