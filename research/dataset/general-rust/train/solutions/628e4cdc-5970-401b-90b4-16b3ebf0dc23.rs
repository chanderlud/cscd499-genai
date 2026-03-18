use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn merge_unique_with_coverage(streams: Vec<Vec<i64>>) -> Vec<(i64, usize)> {
    let mut result = Vec::new();
    let mut heap = BinaryHeap::new();
    let mut indices = vec![0; streams.len()];

    // Initialize heap with first element from each non-empty stream
    for (i, stream) in streams.iter().enumerate() {
        if let Some(&value) = stream.first() {
            heap.push(Reverse((value, i)));
        }
    }

    while let Some(Reverse((current_value, stream_idx))) = heap.pop() {
        let mut coverage = 1;
        let mut streams_to_advance = vec![stream_idx];

        // Collect all streams with the same current value
        while let Some(&Reverse((value, idx))) = heap.peek() {
            if value == current_value {
                heap.pop();
                coverage += 1;
                streams_to_advance.push(idx);
            } else {
                break;
            }
        }

        // Advance each stream past current_value and its duplicates
        for idx in streams_to_advance {
            let stream = &streams[idx];
            let mut pos = indices[idx];

            // Skip all occurrences of current_value in this stream
            while pos < stream.len() && stream[pos] <= current_value {
                pos += 1;
            }

            indices[idx] = pos;

            // If stream has more elements, push next value to heap
            if pos < stream.len() {
                heap.push(Reverse((stream[pos], idx)));
            }
        }

        result.push((current_value, coverage));
    }

    result
}
