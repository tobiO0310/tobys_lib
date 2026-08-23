use tobys_lib_macros::comprehension;

fn main() {
    let vec1 = vec![1, 2, 3];
    let vec2: Vec<_> = comprehension![x for x in &vec1].copied().collect();
    assert_eq!(vec1, vec2)
}