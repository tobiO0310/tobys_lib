use tobys_lib_macros::comprehension;

fn main() {
    let vectors = vec![vec![1, 2, 3], vec![4, 5, 6]];
    comprehension![x for vector in vectors for x in vector];
}
