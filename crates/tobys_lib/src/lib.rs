//! A little library full of weird little functions/structs/macros.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]

mod alias;
pub mod numbers;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub mod macros {
    //! A few custom macros that can simply development, see more in [`tobys_lib_macros`].

    #[doc(inline)]
    pub use tobys_lib_macros::*;
}
