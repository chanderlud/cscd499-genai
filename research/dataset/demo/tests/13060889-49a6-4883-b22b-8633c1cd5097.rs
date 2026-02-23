#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_edge_coupon() {
        let n = 2;
        let edges = vec![(0, 1, 10)];
        assert_eq!(min_cost_with_coupon(n, &edges), Some(5));
    }

    #[test]
    fn coupon_optional_not_required() {
        // Best path is already cheap; coupon doesn't improve it.
        // 0->2 costs 1, while 0->1->2 costs 100+100.
        let n = 3;
        let edges = vec![(0, 2, 1), (0, 1, 100), (1, 2, 100)];
        assert_eq!(min_cost_with_coupon(n, &edges), Some(1));
    }

    #[test]
    fn coupon_choice_matters_across_paths() {
        // Two candidate routes:
        // A: 0->1->3 costs 8+8 = 16, coupon on one edge => 4+8 = 12
        // B: 0->2->3 costs 3+20 = 23, coupon on 20 => 3+10 = 13
        // Best is route A with coupon: 12
        let n = 4;
        let edges = vec![(0, 1, 8), (1, 3, 8), (0, 2, 3), (2, 3, 20)];
        assert_eq!(min_cost_with_coupon(n, &edges), Some(12));
    }

    #[test]
    fn unreachable_returns_none() {
        let n = 4;
        let edges = vec![(0, 1, 5), (1, 0, 5), (2, 3, 1)];
        assert_eq!(min_cost_with_coupon(n, &edges), None);
    }

    #[test]
    fn odd_cost_uses_floor_division() {
        // 0->1->2 costs 5+1 = 6
        // coupon on 5 => floor(5/2)=2, total 3 (best)
        let n = 3;
        let edges = vec![(0, 1, 5), (1, 2, 1)];
        assert_eq!(min_cost_with_coupon(n, &edges), Some(3));
    }

    #[test]
    fn multiple_edges_and_cycle() {
        // Multiple edges 0->1 with different costs and a cycle 1->0.
        // Best: take 0->1 with cost 10 (coupon => 5), then 1->2 cost 1 => total 6.
        let n = 3;
        let edges = vec![
            (0, 1, 100),
            (0, 1, 10),
            (1, 0, 1),
            (1, 2, 1),
        ];
        assert_eq!(min_cost_with_coupon(n, &edges), Some(6));
    }
}