//! The

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod alias;
#[cfg(all(feature = "cron", any(feature = "alloc", feature = "std")))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(feature = "cron", any(feature = "alloc", feature = "std"))))
)]
pub mod cron;
