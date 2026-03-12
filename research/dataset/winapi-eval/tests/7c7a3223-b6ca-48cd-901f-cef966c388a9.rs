#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static IOCP_ECHO_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_lock() -> MutexGuard<'static, ()> {
        IOCP_ECHO_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn round_trip(payload: &[u8]) {
        let out = iocp_tcp_echo(payload).unwrap_or_else(|e| {
            panic!(
                "iocp_tcp_echo failed for {}-byte payload: {e}",
                payload.len()
            )
        });

        assert_eq!(
            out.len(),
            payload.len(),
            "echo length mismatch: sent {} bytes, got {} bytes",
            payload.len(),
            out.len()
        );
        assert_eq!(out, payload, "echoed bytes differed from input");
    }

    fn patterned_payload(len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| match i % 8 {
                0 => 0x00,
                1 => 0xFF,
                2 => b'\r',
                3 => b'\n',
                _ => ((i * 31 + 17) % 251) as u8,
            })
            .collect()
    }

    #[test]
    fn echoes_small_ascii_payload() {
        let _guard = test_lock();
        round_trip(b"hello over iocp");
    }

    #[test]
    fn echoes_binary_payload_with_nuls() {
        let _guard = test_lock();

        let payload = [
            0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF, b'h', 0x00, b'i', b'\r', b'\n', 0x00, b'!',
        ];

        round_trip(&payload);
    }

    #[test]
    fn echoes_large_payload_across_multiple_buffers() {
        let _guard = test_lock();

        // Large enough to force multiple send/recv completions in most implementations,
        // while still being fast and deterministic.
        let payload = patterned_payload(128 * 1024 + 31);

        round_trip(&payload);
    }

    #[test]
    fn can_be_called_repeatedly_without_leaking_state() {
        let _guard = test_lock();

        for payload in [
            b"first".as_slice(),
            b"second payload".as_slice(),
            b"third payload with more bytes".as_slice(),
        ] {
            round_trip(payload);
        }

        let payload = patterned_payload(32 * 1024 + 3);
        round_trip(&payload);
    }
}