#[cfg(test)]
mod tests {
    use tobys_lib_macros::comprehension;

    #[test]
    fn baseline_test() {
        let vec = vec![1, 2, 3];
        let res: Vec<_> = comprehension![x for x in &vec].copied().collect();
        assert_eq!(vec, res);
    }

    #[test]
    fn multiple_for_clauses() {
        let vectors = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let res: Vec<_> =
            comprehension![x for x in vec if x & 1 == 1 for vec in vectors]
                .collect();
        assert_eq!(res, vec![1, 3, 5]);
    }

    struct Color(i32, i32, i32);
    #[test]
    fn pattern_destructuring() {
        let vec = vec![
            Color(255, 255, 255),
            Color(0, 0, 0),
            Color(255, 0, 0),
            Color(0, 255, 0),
            Color(0, 0, 255),
        ];
        let vec: Vec<_> =
            comprehension![(r, g, b) for Color(r, g, b) in vec if r != 0]
                .collect();
        assert_eq!(vec, vec![(255, 255, 255), (255, 0, 0)]);
    }

    #[test]
    fn multiple_if_clauses() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let res: Vec<_> =
            comprehension![x for x in vec if x & 1 != 0 && x % 3 != 0]
                .collect();
        assert_eq!(res, vec![1, 5, 7]);
    }
}
