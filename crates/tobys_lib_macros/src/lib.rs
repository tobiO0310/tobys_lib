//! The macro sub-crate for my little library.
//!
//! See each macro for what they do~

//#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

mod python;
pub(crate) mod utilities;

/// Python comprehension syntax in rust.
///
/// In python, there exists an expression known as a [comprehension].
/// I personally really like this syntax, and would like to use it in Rust.
/// The code is primarily taken from [Logan Smith]'s `YouTube` video [Comprehending Proc Macros].
/// I would invite you to watch it, as it is really informative~
///
/// # Examples
///
/// 1) Multiply all items in a vector by a number
/// ```rust
/// # use tobys_lib_macros::comprehension;
/// let vec = vec![1, 2, 3];
/// let updated: Vec<_> = comprehension![x * 3 for x in vec].collect();
/// assert_eq!(updated, vec![3, 6, 9]);
/// ```
/// 2) Get all numbers that are even in a list
/// ```rust
/// # use tobys_lib_macros::comprehension;
/// let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let updated: Vec<_> = comprehension![x for x in vec if x & 1 == 0].collect();
/// assert_eq!(updated, vec![2, 4, 6, 8, 10]);
/// ```
/// 3) Flatten a map, but delete even numbers
/// ```rust
/// # use tobys_lib_macros::comprehension;
/// let vectors = vec![vec![1, 2, 3], vec![4, 5, 6]];
/// let vec: Vec<_> = comprehension![x for x in vec if x & 1 == 1 for vec in vectors].collect();
/// assert_eq!(vec, vec![1, 3, 5]);
/// ```
///
/// [comprehension]: https://docs.python.org/3/reference/expressions.html#displays-for-lists-sets-and-dictionaries
/// [Logan Smith]: https://www.youtube.com/@_noisecode
/// [Comprehending Proc Macros]: https://www.youtube.com/watch?v=SMCRQj9Hbx8
#[proc_macro]
pub fn comprehension(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    python::comprehension::comprehension_impl(input)
}
