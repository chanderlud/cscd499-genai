#[cfg(all(test, windows))]
mod tests {
    use super::align_mapping_offset;

    // Windows API types must come from windows crate v0.62.2
    use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

    fn system_allocation_granularity() -> u32 {
        unsafe {
            let mut info = SYSTEM_INFO::default();
            GetSystemInfo(&mut info);
            info.dwAllocationGranularity
        }
    }

    fn expected_alignment(offset: u64, g: u64) -> (u64, u64) {
        // Aligned base is the greatest multiple of g <= offset
        let aligned = offset - (offset % g);
        let delta = offset - aligned;
        (aligned, delta)
    }

    #[test]
    fn uses_real_system_granularity_and_basic_invariants_hold() {
        let sys_g = system_allocation_granularity();
        assert!(sys_g > 0, "dwAllocationGranularity should never be 0 on Windows");

        let (aligned, delta, g) = align_mapping_offset(0).unwrap();

        // Must match the real system call.
        assert_eq!(g, sys_g);

        // Invariants
        let g64 = g as u64;
        assert_eq!(aligned + delta, 0);
        assert_eq!(aligned % g64, 0);
        assert!(delta < g64);
        assert!(aligned <= 0);
    }

    #[test]
    fn aligns_correctly_for_edge_case_offsets() {
        let sys_g = system_allocation_granularity();
        assert!(sys_g > 0, "dwAllocationGranularity should never be 0 on Windows");
        let g64 = sys_g as u64;

        let mut offsets = Vec::new();
        offsets.push(0u64);
        offsets.push(1u64);

        if g64 > 1 {
            offsets.push(g64 - 1);
        }
        offsets.push(g64);
        offsets.push(g64 + 1);
        offsets.push(1_234_567);

        // High end of u64 space
        offsets.push(u64::MAX);
        offsets.push(u64::MAX - 1);
        offsets.push(u64::MAX - (g64 - 1)); // ensures a case where delta should be g-1

        for offset in offsets {
            let (aligned, delta, g) = align_mapping_offset(offset).unwrap();

            // Must match the real Windows granularity
            assert_eq!(g, sys_g);

            let (exp_aligned, exp_delta) = expected_alignment(offset, g64);

            // Spec: alignment is non-negotiable.
            assert_eq!(aligned, exp_aligned, "aligned base mismatch for offset={offset}");
            assert_eq!(delta, exp_delta, "delta mismatch for offset={offset}");

            // Core invariants
            assert_eq!(aligned + delta, offset, "reconstruction failed for offset={offset}");
            assert_eq!(aligned % g64, 0, "base not aligned for offset={offset}");
            assert!(delta < g64, "delta must be < granularity for offset={offset}");
            assert!(aligned <= offset, "aligned base must be <= offset for offset={offset}");
        }
    }

    #[test]
    fn fuzzes_many_offsets_without_overflow_or_invariant_breaks() {
        let sys_g = system_allocation_granularity();
        assert!(sys_g > 0, "dwAllocationGranularity should never be 0 on Windows");
        let g64 = sys_g as u64;

        // Deterministic pseudo-random walk across u64 space (no extra deps).
        let mut x = 0x1234_5678_9abc_def0u64;

        for _ in 0..10_000 {
            x = x
                .wrapping_mul(6364136223846793005u64)
                .wrapping_add(1442695040888963407u64);

            let offset = x;

            let (aligned, delta, g) = align_mapping_offset(offset).unwrap();
            assert_eq!(g, sys_g);

            // Expected math
            let (exp_aligned, exp_delta) = expected_alignment(offset, g64);
            assert_eq!(aligned, exp_aligned);
            assert_eq!(delta, exp_delta);

            // Invariants
            assert_eq!(aligned + delta, offset);
            assert_eq!(aligned % g64, 0);
            assert!(delta < g64);
            assert!(aligned <= offset);
        }
    }
}
