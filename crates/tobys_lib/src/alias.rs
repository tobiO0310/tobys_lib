#![allow(
    unused_imports,
    reason = "all imports in here is enabled on at least one feature"
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::vec;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
pub use ::core::iter::repeat_with;
#[cfg(feature = "std")]
pub use ::std::iter::repeat_with;
#[cfg(feature = "std")]
pub use ::std::vec;
#[cfg(feature = "std")]
pub use ::std::vec::Vec;
