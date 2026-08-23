use tobys_lib_macros::comprehension;

fn main() {
    let vecs = vec![vec![1, 2, 3], vec![4, 5, 6]];
    comprehension![x for vec in vecs for x in vec];
}
