#![allow(
    unused_imports,
    reason = "all imports in here is enabled on at least one feature"
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
mod lib {
    extern crate alloc;

    pub use alloc::{boxed::Box, collections::VecDeque, vec, vec::Vec};
    pub use core::iter::repeat_with;

    pub use hashbrown::{HashMap, hash_map::Entry};
}

#[cfg(feature = "std")]
mod lib {
    pub use std::{
        boxed::Box,
        collections::{HashMap, VecDeque, hash_map::Entry},
        iter::repeat_with,
        vec,
        vec::Vec,
    };
}

pub use lib::*;
