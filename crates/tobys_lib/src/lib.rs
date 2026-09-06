//! A little library full of weird little functions/structs/macros.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod alias;
pub mod numbers;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[doc(inline)]
pub use tobys_lib_macros::*;

#[cfg(all(feature = "cron", any(feature = "alloc", feature = "std")))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "cron", any(feature = "alloc", feature = "std"))))
)]
pub mod cron {
    //! Cron scheduling structs and functions.
    //!
    //! ## Example
    //!
    //! ## Macro
    //!
    //! If `macros` feature is enabled, two macros are available;
    //! - [`create_cron_time`] macro is available to test cron time at compile-time.
    //! - [`create_cron_jobs`] macro is available to create multiple jobs at the same time.
    //!
    //!
    //! [`create_cron_time`]: crate::create_cron_time
    //! [`create_cron_jobs`]: crate::create_cron_jobs
    #[doc(inline)]
    pub use tobys_lib_core::cron::*;
}
