use tobys_lib_macros::comprehension;

fn main() {
    let vec = vec![1, 2, 3];
    comprehension![x for x vec];
}