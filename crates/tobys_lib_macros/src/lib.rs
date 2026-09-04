//! The macro sub-crate for my little library.
//!
//! See each macro for what they do~

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

#[cfg(feature = "cron")]
#[cfg_attr(docsrs, doc(cfg(feature = "cron")))]
mod cron;
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

/// A macro that checks and makes sure the cron input is correct.
/// 
/// This macro will move runtime cron string compilation to compile-time.
/// The compiler will reject any invalid cron jobs, so you are guaranteed your cron string is correct.
/// 
/// The format for this macro is a standard cron job, but not as a string literal.
///
/// # Examples
///
/// 1) Create a cron time that represents every-minute
/// ```
/// # use tobys_lib_macros::create_cron_time;
/// # use tobys_lib::cron::CronTime;
/// let every_minute = create_cron_time!(* * * * *);
/// # assert_eq!(every_minute, CronTime::new("* * * * *").unwrap())
/// ```
/// 
/// 2) Create a cron time that represents every friday the 13th
/// ```
/// # use tobys_lib_macros::create_cron_time;
/// # use tobys_lib::cron::CronTime;
/// let every_minute = create_cron_time!(* * 13 * 5);
/// # assert_eq!(every_minute, CronTime::new("* * 13 * 5").unwrap())
/// ```
/// 
/// 3) Create a cron time that represents every wednesday, saturday, and sunday at 2 am.
/// ```
/// # use tobys_lib_macros::create_cron_time;
/// # use tobys_lib::cron::CronTime;
/// let every_minute = create_cron_time!(0 2 * * */3);
/// # assert_eq!(every_minute, CronTime::new("0 2 * * */3").unwrap())
/// ```
#[cfg(feature = "cron")]
#[cfg_attr(docsrs, doc(cfg(feature = "cron")))]
#[proc_macro]
pub fn create_cron_time(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    cron::create_time_impl(input)
}

/// A macro for creating multiple Cron Jobs.
///
/// It runs compile-time cron schedule verification,
/// so you don't have to worry about checking at runtime.
///
/// # Examples
///
/// Create 3 jobs that happens everyday at 8:00 (8 am), 12:00 (12 pm), and 16:00 (4 pm).
/// ```
/// # use tobys_lib_macros::create_cron_jobs;
/// let jobs = create_cron_jobs!(
///     0 8 * * *, || {
///         Box::pin(async {
///             // do work here
///         })
///     }; 0 12 * * *, || {
///         Box::pin(async {
///             // do other work here
///         })
///     }; 0 16 * * *, || {
///         Box::pin(async {
///             // do last work here
///         })
///     }
/// );
/// # assert_eq!(jobs.len(), 3);
/// ```
/// 
/// Create a single job, that fires once every blue moon;
/// every january 1st that is a monday at 8:00 (8 am).
/// ```
/// # use tobys_lib_macros::create_cron_jobs;
/// let jobs = create_cron_jobs!(
///     0 8 1 1 1, || {
///         Box::pin(async {
///             // do work here
///         })
///     };
/// );
/// # assert_eq!(jobs.len(), 1);
/// ```
#[cfg(feature = "cron")]
#[cfg_attr(docsrs, doc(cfg(feature = "cron")))]
#[proc_macro]
pub fn create_cron_jobs(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    cron::create_jobs_impl(input)
}
