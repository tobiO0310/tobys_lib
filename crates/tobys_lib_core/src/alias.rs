#![allow(
    unused_imports,
    reason = "all imports in here is enabled on at least one feature"
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::{
    boxed::Box,
    format,
    vec::{self, Vec},
};

#[cfg(feature = "std")]
pub use ::std::{
    boxed::Box,
    format,
    vec::{self, Vec},
};
