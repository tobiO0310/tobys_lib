use tobys_lib_macros::comprehension;

fn main() {
    let vecs = vec![vec![1, 2, 3], vec![4, 5, 6]];
    let vec: Vec<_> = comprehension![x for x in vec for vec in vecs].collect();
    assert_eq!(vec, vec![1, 2, 3, 4, 5, 6]);
}
