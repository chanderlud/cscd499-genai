#[cfg(all(test, windows))]
mod winsock_tcp_reverse_echo_tests {
    use super::winsock_tcp_reverse_echo;

    fn expected(payload: &[u8]) -> Vec<u8> {
        let mut v = payload.to_vec();
        v.reverse();
        v
    }

    /// Tiny deterministic PRNG so we don't need external deps in tests.
    /// (Yes, it's bad. That's the point: it's stable and repeatable.)
    fn gen_bytes(len: usize, seed: u64) -> Vec<u8> {
        let mut s = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            // xorshift-ish
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            out.push((s & 0xFF) as u8);
        }
        out
    }

    #[test]
    fn empty_payload_returns_empty() -> std::io::Result<()> {
        let out = winsock_tcp_reverse_echo(&[])?;
        assert!(out.is_empty());
        Ok(())
    }

    #[test]
    fn single_byte_round_trips() -> std::io::Result<()> {
        let out = winsock_tcp_reverse_echo(&[0xAB])?;
        assert_eq!(out, vec![0xAB]);
        Ok(())
    }

    #[test]
    fn small_ascii_example() -> std::io::Result<()> {
        let out = winsock_tcp_reverse_echo(b"abcd")?;
        assert_eq!(out, b"dcba");
        Ok(())
    }

    #[test]
    fn preserves_binary_and_nuls() -> std::io::Result<()> {
        let payload = [0x00, 0xFF, 0x01, 0x80, 0x7F, 0x00, 0x10, 0x00, 0xEE, 0xDD];
        let out = winsock_tcp_reverse_echo(&payload)?;
        assert_eq!(out, expected(&payload));
        Ok(())
    }

    #[test]
    fn does_not_mutate_input_buffer() -> std::io::Result<()> {
        let mut payload = vec![1u8, 2, 3, 4, 5];
        let out = winsock_tcp_reverse_echo(&payload)?;
        assert_eq!(out, vec![5, 4, 3, 2, 1]);

        // Mutate after the call; output must stay the same (obviously, but tests exist for a reason).
        payload[0] = 99;
        assert_eq!(out, vec![5, 4, 3, 2, 1]);
        Ok(())
    }

    #[test]
    fn large_payload_requires_multiple_recv_loops() -> std::io::Result<()> {
        // Big enough to practically force chunking in recv/send implementations.
        // Also not a nice round number, because edge cases are allergic to round numbers.
        let payload = gen_bytes(256 * 1024 + 3, 12345);
        let out = winsock_tcp_reverse_echo(&payload)?;
        assert_eq!(out.len(), payload.len());
        assert_eq!(out, expected(&payload));
        Ok(())
    }

    #[test]
    fn many_varied_payloads_property_check() -> std::io::Result<()> {
        // A small “property test” without pulling in proptest/quickcheck.
        for i in 0..64u64 {
            let len = ((i * 37 + 11) % 4096) as usize; // 0..4095 with variety
            let payload = gen_bytes(len, 0xBADC0FFEEu64 ^ i);
            let out = winsock_tcp_reverse_echo(&payload)?;
            assert_eq!(out, expected(&payload), "mismatch at i={i}, len={len}");
        }
        Ok(())
    }

    #[test]
    fn repeated_calls_dont_get_confused() -> std::io::Result<()> {
        // Catches “forgot to close socket”, “WSAStartup/WSACleanup imbalance”, and other classics.
        for i in 0..50u64 {
            let payload = gen_bytes(((i * 101) % 8192) as usize, i ^ 0xDEADBEEF);
            let out = winsock_tcp_reverse_echo(&payload)?;
            assert_eq!(out, expected(&payload), "mismatch on iteration {i}");
        }
        Ok(())
    }

    #[test]
    fn concurrent_calls_work() {
        // If the implementation has global state problems (hello, WSAStartup/WSACleanup),
        // this can make it show its teeth.
        let mut handles = Vec::new();

        for t in 0..8u64 {
            handles.push(std::thread::spawn(move || -> std::io::Result<()> {
                let payload = gen_bytes(32 * 1024 + (t as usize * 17), 0xCAFEBABE ^ t);
                let out = winsock_tcp_reverse_echo(&payload)?;
                assert_eq!(out, expected(&payload));
                Ok(())
            }));
        }

        for (idx, h) in handles.into_iter().enumerate() {
            match h.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => panic!("thread {idx} returned io error: {e}"),
                Err(_) => panic!("thread {idx} panicked"),
            }
        }
    }
}
