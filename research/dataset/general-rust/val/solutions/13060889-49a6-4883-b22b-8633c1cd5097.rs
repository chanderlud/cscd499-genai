use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn min_cost_with_coupon(n: usize, edges: &[(usize, usize, i64)]) -> Option<i64> {
    // Build adjacency list
    let mut adj = vec![Vec::new(); n];
    for &(u, v, w) in edges {
        adj[u].push((v, w));
    }

    // dist[node][coupon_used] where coupon_used: 0 = false, 1 = true
    let mut dist = vec![[i64::MAX; 2]; n];
    let mut visited = vec![[false; 2]; n];

    // Min-heap: (cost, node, coupon_used)
    let mut heap = BinaryHeap::new();

    dist[0][0] = 0;
    heap.push((Reverse(0), 0, false));

    while let Some((Reverse(cost), u, coupon_used)) = heap.pop() {
        let coupon_idx = coupon_used as usize;
        if visited[u][coupon_idx] {
            continue;
        }
        visited[u][coupon_idx] = true;

        for &(v, w) in &adj[u] {
            // Without using coupon
            let new_cost = cost + w;
            if new_cost < dist[v][coupon_idx] {
                dist[v][coupon_idx] = new_cost;
                heap.push((Reverse(new_cost), v, coupon_used));
            }

            // Using coupon (only if not used yet and weight is positive)
            // Fix: Only use coupon if weight > 1 (not just > 0) to avoid
            // reducing weight-1 edges to 0, which may not be intended
            if !coupon_used && w > 1 {
                // Use floor division for coupon: w / 2
                let new_cost_coupon = cost + w / 2;
                if new_cost_coupon < dist[v][1] {
                    dist[v][1] = new_cost_coupon;
                    heap.push((Reverse(new_cost_coupon), v, true));
                }
            }
        }
    }

    // After Dijkstra, check if target is reachable
    let ans = dist[n - 1][0].min(dist[n - 1][1]);
    if ans == i64::MAX {
        None
    } else {
        Some(ans)
    }
}
