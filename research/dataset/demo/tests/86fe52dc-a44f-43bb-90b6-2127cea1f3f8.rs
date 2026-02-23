#[cfg(test)]
mod tests {
    use super::*;

    const MOD: i64 = 1_000_000_007;

    #[test]
    fn single_road_success() {
        let n = 2;
        let roads = vec![(0, 1, 'h')];
        let portals: Vec<(usize, usize)> = vec![];
        let rewinds: Vec<(usize, usize)> = vec![];
        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 1, "h");
        assert_eq!(steps, 1);
        assert_eq!(ways, 1);
    }

    #[test]
    fn two_shortest_ways() {
        // 0 -a-> 1 -b-> 3
        // 0 -a-> 2 -b-> 3
        let n = 4;
        let roads = vec![(0, 1, 'a'), (0, 2, 'a'), (1, 3, 'b'), (2, 3, 'b')];
        let portals: Vec<(usize, usize)> = vec![];
        let rewinds: Vec<(usize, usize)> = vec![];
        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 3, "ab");
        assert_eq!(steps, 2);
        assert_eq!(ways, 2);
    }

    #[test]
    fn portal_needed_to_reach_matching_road() {
        // Need spell "b".
        // Road 0->1 is 'a' (not usable at progress 0), but portal 0->1 works.
        // Then road 1->2 is 'b'.
        let n = 3;
        let roads = vec![(0, 1, 'a'), (1, 2, 'b')];
        let portals = vec![(0, 1)];
        let rewinds: Vec<(usize, usize)> = vec![];
        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 2, "b");
        assert_eq!(steps, 2);
        assert_eq!(ways, 1);
    }

    #[test]
    fn rewind_trap_makes_it_impossible() {
        // spell "ab"
        // 0 -a-> 1 (entering 1 rewinds 1, so progress goes back to 0)
        // 1 -b-> 2 cannot be taken because next needed is still 'a'
        let n = 3;
        let roads = vec![(0, 1, 'a'), (1, 2, 'b')];
        let portals: Vec<(usize, usize)> = vec![];
        let rewinds = vec![(1, 1)];
        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 2, "ab");
        assert_eq!((steps, ways), (-1, 0));
    }

    #[test]
    fn goal_rewind_can_prevent_completion() {
        // spell "a"
        // 0 -a-> 1 would normally finish, but entering goal rewinds 1 -> progress becomes 0
        let n = 2;
        let roads = vec![(0, 1, 'a')];
        let portals: Vec<(usize, usize)> = vec![];
        let rewinds = vec![(1, 1)];
        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 1, "a");
        assert_eq!((steps, ways), (-1, 0));
    }

    #[test]
    fn empty_spell_only_succeeds_if_start_is_goal() {
        let n = 3;
        let roads: Vec<(usize, usize, char)> = vec![(0, 1, 'a'), (1, 2, 'b')];
        let portals: Vec<(usize, usize)> = vec![(0, 2)];
        let rewinds: Vec<(usize, usize)> = vec![(2, 5)];

        let (steps1, ways1) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 0, "");
        assert_eq!(steps1, 0);
        assert_eq!(ways1, 1);

        let (steps2, ways2) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 2, "");
        assert_eq!((steps2, ways2), (-1, 0));
    }

    #[test]
    fn many_ways_modulo_smoke_test() {
        // Build a small layered diamond that creates 8 shortest ways for spell "abc".
        //
        // layer0: 0
        // layer1: 1,2 (a)
        // layer2: 3,4 (b) from both 1 and 2
        // layer3: 5,6 (c) from both 3 and 4
        // goal: 7 via portals (no consume) from 5 and 6, but portal still costs 1 step
        //
        // Shortest consumes 3 letters in 3 road steps, then 1 portal step => 4 steps total.
        // Ways: 2 * 2 * 2 = 8, then choose portal from 5 or 6 is already determined by last node (still 8).
        let n = 8;
        let mut roads = vec![
            (0, 1, 'a'), (0, 2, 'a'),
            (1, 3, 'b'), (1, 4, 'b'),
            (2, 3, 'b'), (2, 4, 'b'),
            (3, 5, 'c'), (3, 6, 'c'),
            (4, 5, 'c'), (4, 6, 'c'),
        ];
        let portals = vec![(5, 7), (6, 7)];
        let rewinds: Vec<(usize, usize)> = vec![];

        let (steps, ways) = time_rewind_courier(n, &roads, &portals, &rewinds, 0, 7, "abc");
        assert_eq!(steps, 4);
        assert_eq!(ways % MOD, 8);
    }
}