
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row() {
        assert!(
            get_row(
                vec![
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 1, 6],
                    vec![1, 2, 3, 4, 5, 1]
                ],
                1
            ) == vec![vec![0, 0], vec![1, 0], vec![1, 4], vec![2, 0], vec![2, 5]]
        );
        assert!(
            get_row(
                vec![
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6]
                ],
                2
            ) == vec![
                vec![0, 1],
                vec![1, 1],
                vec![2, 1],
                vec![3, 1],
                vec![4, 1],
                vec![5, 1]
            ]
        );
        assert!(
            get_row(
                vec![
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 2, 3, 4, 5, 6],
                    vec![1, 1, 3, 4, 5, 6],
                    vec![1, 2, 1, 4, 5, 6],
                    vec![1, 2, 3, 1, 5, 6],
                    vec![1, 2, 3, 4, 1, 6],
                    vec![1, 2, 3, 4, 5, 1]
                ],
                1
            ) == vec![
                vec![0, 0],
                vec![1, 0],
                vec![2, 0],
                vec![2, 1],
                vec![3, 0],
                vec![3, 2],
                vec![4, 0],
                vec![4, 3],
                vec![5, 0],
                vec![5, 4],
                vec![6, 0],
                vec![6, 5]
            ]
        );
        let v: Vec<Vec<i32>> = vec![];
        assert!(get_row(vec![], 1) == v);
        assert!(get_row(vec![vec![1]], 2) == v);
        assert!(get_row(vec![vec![], vec![1], vec![1, 2, 3]], 3) == vec![vec![2, 2]]);
    }

}
