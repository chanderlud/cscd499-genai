#[cfg(test)]
mod tests {
    use super::merge_unique_with_coverage;

    #[test]
    fn empty_input() {
        let streams: Vec<Vec<i64>> = vec![];
        let out = merge_unique_with_coverage(streams);
        assert!(out.is_empty());
    }

    #[test]
    fn all_streams_empty() {
        let streams = vec![vec![], vec![], vec![]];
        let out = merge_unique_with_coverage(streams);
        assert!(out.is_empty());
    }

    #[test]
    fn single_stream_with_duplicates() {
        let streams = vec![vec![1, 1, 2, 2, 2, 5]];
        let out = merge_unique_with_coverage(streams);
        assert_eq!(out, vec![(1, 1), (2, 1), (5, 1)]);
    }

    #[test]
    fn multiple_streams_overlap_and_internal_dupes() {
        let streams = vec![
            vec![1, 1, 3, 4, 4, 10],
            vec![1, 2, 2, 4, 9],
            vec![2, 2, 2, 3, 3, 8, 10, 10],
        ];
        let out = merge_unique_with_coverage(streams);

        // Values:
        // 1 appears in streams 0,1 => coverage 2
        // 2 appears in streams 1,2 => coverage 2
        // 3 appears in streams 0,2 => coverage 2
        // 4 appears in streams 0,1 => coverage 2
        // 8 appears in stream 2 => coverage 1
        // 9 appears in stream 1 => coverage 1
        // 10 appears in streams 0,2 => coverage 2
        assert_eq!(
            out,
            vec![(1, 2), (2, 2), (3, 2), (4, 2), (8, 1), (9, 1), (10, 2)]
        );
    }

    #[test]
    fn negative_values_and_large_range() {
        let streams = vec![
            vec![-10, -10, -5, 0, 7],
            vec![-6, -5, -5, 7, 7, 9],
            vec![-10, -6, -6, 100],
        ];
        let out = merge_unique_with_coverage(streams);

        // -10 in streams 0,2 => 2
        // -6  in streams 1,2 => 2
        // -5  in streams 0,1 => 2
        // 0   in stream 0 => 1
        // 7   in streams 0,1 => 2
        // 9   in stream 1 => 1
        // 100 in stream 2 => 1
        assert_eq!(
            out,
            vec![(-10, 2), (-6, 2), (-5, 2), (0, 1), (7, 2), (9, 1), (100, 1)]
        );
    }

    #[test]
    fn identical_streams() {
        let streams = vec![
            vec![1, 2, 2, 3],
            vec![1, 1, 2, 3, 3],
            vec![1, 2, 3],
        ];
        let out = merge_unique_with_coverage(streams);
        assert_eq!(out, vec![(1, 3), (2, 3), (3, 3)]);
    }

    #[test]
    fn one_long_stream_many_small() {
        let streams = vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![2],
            vec![2, 2],
            vec![9, 9, 9],
            vec![],
            vec![5, 6],
        ];
        let out = merge_unique_with_coverage(streams);

        // 1 in stream0 =>1
        // 2 in stream0,1,2 =>3
        // 3 in stream0 =>1
        // 4 in stream0 =>1
        // 5 in stream0,5 =>2
        // 6 in stream0,5 =>2
        // 7 in stream0 =>1
        // 8 in stream0 =>1
        // 9 in stream0,3 =>2
        assert_eq!(
            out,
            vec![(1,1),(2,3),(3,1),(4,1),(5,2),(6,2),(7,1),(8,1),(9,2)]
        );
    }
}