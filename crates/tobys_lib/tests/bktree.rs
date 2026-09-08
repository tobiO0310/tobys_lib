#![expect(clippy::tests_outside_test_module)]

#[test]
fn main() {
    #[cfg(feature = "alloc")]
    {
        use tobys_lib::trees::bk_tree::*;

        let tree = BkTree::new_with_items(
            levenshtein,
            vec!["She", "sells", "sea", "shells"],
        );
        let mut vec = tree.find(&"sells", 3);
        println!("{vec:?}");
        vec.sort_by_key(|(a, _)| *a);
    }
}
