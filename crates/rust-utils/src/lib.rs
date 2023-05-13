#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

mod core_;
pub use self::core_::*;

#[cfg(feature = "alloc")]
cfg_group! {
    mod alloc_;
    pub use self::alloc_::*;
}
