#[cfg(test)]
mod tests {
    use super::*;

    const MOD: u64 = 1_000_000_007;

    #[test]
    fn basic_two_hop_beats_direct() {
        // 0 -> 1 -> 2 arrives in 5, direct arrives in 10
        let n = 3;
        let edges = vec![
            (0, 1, 2, 0, 0),
            (1, 2, 3, 0, 0),
            (0, 2, 10, 0, 0),
        ];
        let got = chrono_transit(n, &edges, 0, 2);
        assert_eq!(got, Some((5, 1, vec![0, 1, 2])));
    }

    #[test]
    fn slack_intermediate_arrival_still_counts() {
        // Two optimal routes to 2 both arrive at time 11:
        // A: 0->1 (t=1), wait to depart at 10, arrive 11
        // B: 0->3 (t=5), 3->1 (t=6), wait to depart at 10, arrive 11
        // Canonical prefers fewer rides, so route A is chosen.
        let n = 4;
        let edges = vec![
            (0, 1, 1, 0, 0),
            (0, 3, 5, 0, 0),
            (3, 1, 1, 0, 0),
            (1, 2, 1, 10, 10),
        ];
        let got = chrono_transit(n, &edges, 0, 2);
        assert_eq!(got, Some((11, 2, vec![0, 1, 2])));
    }

    #[test]
    fn canonical_prefers_less_waiting_over_lex() {
        // Two optimal routes arrive at time 6, same rides (2), different waiting:
        // 0->1 arrives 1, next dep 5, wait 4, arrive 6
        // 0->2 arrives 1, next dep 2, wait 1, arrive 6
        // Canonical picks via 2 due to less total waiting, even though [0,1,3] is lex smaller.
        let n = 4;
        let edges = vec![
            (0, 1, 1, 0, 0),
            (0, 2, 1, 0, 0),
            (1, 3, 1, 5, 10),
            (2, 3, 4, 2, 10),
        ];
        let got = chrono_transit(n, &edges, 0, 3);
        assert_eq!(got, Some((6, 2, vec![0, 2, 3])));
    }

    #[test]
    fn canonical_lex_tie_breaker() {
        // Two optimal routes: [0,1,3] and [0,2,3], same arrival, rides, waiting.
        // Canonical picks lexicographically smaller node sequence: [0,1,3].
        let n = 4;
        let edges = vec![
            (0, 1, 1, 0, 0),
            (0, 2, 1, 0, 0),
            (1, 3, 1, 5, 10),
            (2, 3, 1, 5, 10),
        ];
        let got = chrono_transit(n, &edges, 0, 3);
        assert_eq!(got, Some((6, 2, vec![0, 1, 3])));
    }

    #[test]
    fn unreachable_returns_none() {
        let n = 3;
        let edges = vec![(0, 1, 1, 0, 0)];
        let got = chrono_transit(n, &edges, 0, 2);
        assert_eq!(got, None);
    }

    #[test]
    fn counts_more_than_two_and_picks_shortest_rides() {
        // Optimal arrival time is 11.
        // 3 direct-to-target-layer routes (2 rides each): 0-1-4, 0-2-4, 0-3-4
        // Plus one longer route still optimal due to schedule slack: 0-2-1-4 (3 rides)
        // Total optimal routes: 4. Canonical chooses 0-1-4 (fewest rides; ties resolved lex).
        let n = 5;
        let edges = vec![
            (0, 1, 1, 0, 0),
            (0, 2, 1, 0, 0),
            (0, 3, 1, 0, 0),
            (1, 4, 1, 10, 10),
            (2, 4, 1, 10, 10),
            (3, 4, 1, 10, 10),
            (2, 1, 1, 0, 0),
        ];
        let got = chrono_transit(n, &edges, 0, 4);
        assert_eq!(got, Some((11, 4, vec![0, 1, 4])));
    }

    #[test]
    fn modulo_count_large_binary_choices() {
        // Build L steps of binary choices:
        // Choice nodes c[i] = 3*i, option nodes o0[i] = 3*i+1, o1[i] = 3*i+2
        // Edges: c[i]->o0[i], c[i]->o1[i], then each option -> c[i+1], all travel=1, interval=0.
        //
        // Each step doubles number of routes; arrival time = 2*L.
        // Number of optimal routes = 2^L mod MOD.
        // Canonical route picks all o0 options due to lex ordering (o0[i] < o1[i]).
        let l: usize = 40;
        let n = 3 * l + 1;
        let source = 0;
        let target = 3 * l;

        let mut edges: Vec<(usize, usize, u64, u64, u64)> = Vec::with_capacity(4 * l);
        for i in 0..l {
            let ci = 3 * i;
            let o0 = 3 * i + 1;
            let o1 = 3 * i + 2;
            let cnext = 3 * (i + 1);
            edges.push((ci, o0, 1, 0, 0));
            edges.push((ci, o1, 1, 0, 0));
            edges.push((o0, cnext, 1, 0, 0));
            edges.push((o1, cnext, 1, 0, 0));
        }

        let expected_time = (2 * l) as u64;
        let expected_count = mod_pow(2, l as u64, MOD);

        let mut expected_path: Vec<usize> = Vec::with_capacity(2 * l + 1);
        expected_path.push(0);
        for i in 0..l {
            expected_path.push(3 * i + 1);     // o0[i]
            expected_path.push(3 * (i + 1));   // c[i+1]
        }

        let got = chrono_transit(n, &edges, source, target);
        assert_eq!(got, Some((expected_time, expected_count, expected_path)));
    }

    fn mod_pow(mut a: u64, mut e: u64, m: u64) -> u64 {
        let mut r = 1u64 % m;
        a %= m;
        while e > 0 {
            if (e & 1) == 1 {
                r = ((r as u128 * a as u128) % m as u128) as u64;
            }
            a = ((a as u128 * a as u128) % m as u128) as u64;
            e >>= 1;
        }
        r
    }
}