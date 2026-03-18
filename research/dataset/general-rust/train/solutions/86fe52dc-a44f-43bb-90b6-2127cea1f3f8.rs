use std::collections::VecDeque;

const MOD: i64 = 1_000_000_007;

pub fn time_rewind_courier(
    n: usize,
    roads: &[(usize, usize, char)],
    portals: &[(usize, usize)],
    rewinds: &[(usize, usize)],
    start: usize,
    goal: usize,
    spell: &str,
) -> (i32, i64) {
    let spell_len = spell.len();
    let spell_chars: Vec<char> = spell.chars().collect();

    // Precompute rewind amounts for each room
    let mut rewind_amount = vec![0; n];
    for &(room, k) in rewinds {
        rewind_amount[room] = k;
    }

    // Build adjacency lists
    let mut roads_adj = vec![Vec::new(); n];
    for &(u, v, c) in roads {
        roads_adj[u].push((v, c));
    }

    let mut portals_adj = vec![Vec::new(); n];
    for &(u, v) in portals {
        portals_adj[u].push(v);
    }

    // dist[room][progress] = minimum steps to reach (room, progress)
    // ways[room][progress] = number of ways to reach (room, progress) with minimum steps
    let mut dist = vec![vec![-1i32; spell_len + 1]; n];
    let mut ways = vec![vec![0i64; spell_len + 1]; n];

    let mut queue = VecDeque::new();

    // Initialize starting state
    dist[start][0] = 0;
    ways[start][0] = 1;
    queue.push_back((start, 0));

    while let Some((u, i)) = queue.pop_front() {
        let current_dist = dist[u][i];
        let current_ways = ways[u][i];

        // Process roads (consume character)
        for &(v, c) in &roads_adj[u] {
            if i < spell_len && c == spell_chars[i] {
                let new_progress = (i + 1).saturating_sub(rewind_amount[v]);
                let new_dist = current_dist + 1;

                if dist[v][new_progress] == -1 {
                    dist[v][new_progress] = new_dist;
                    ways[v][new_progress] = current_ways;
                    queue.push_back((v, new_progress));
                } else if dist[v][new_progress] == new_dist {
                    ways[v][new_progress] = (ways[v][new_progress] + current_ways) % MOD;
                }
            }
        }

        // Process portals (no character consumption)
        for &v in &portals_adj[u] {
            let new_progress = i.saturating_sub(rewind_amount[v]);
            let new_dist = current_dist + 1;

            if dist[v][new_progress] == -1 {
                dist[v][new_progress] = new_dist;
                ways[v][new_progress] = current_ways;
                queue.push_back((v, new_progress));
            } else if dist[v][new_progress] == new_dist {
                ways[v][new_progress] = (ways[v][new_progress] + current_ways) % MOD;
            }
        }
    }

    // Special case: empty spell
    if spell_len == 0 {
        if start == goal {
            return (0, 1);
        } else {
            return (-1, 0);
        }
    }

    if dist[goal][spell_len] == -1 {
        (-1, 0)
    } else {
        (dist[goal][spell_len], ways[goal][spell_len])
    }
}
