use tobys_lib_macros::comprehension;

fn main() {
    let vec = vec![1, 2, 3];
    comprehension![x rof x in vec];
    comprehension![x for x ni vec];
    comprehension![x rof x ni vec];
    comprehension![x ofr x in vec];
}
