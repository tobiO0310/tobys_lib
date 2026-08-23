//! Test

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod macros {
    //! A few custom macros that can simply development, see more in [`tobys_lib_macros`].

    #[doc(inline)]
    pub use tobys_lib_macros::*;
}
