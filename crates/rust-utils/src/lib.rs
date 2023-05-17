#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(clippy::all)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

mod sealed {
    pub trait Sealed {}
}

mod stdx;
pub use self::stdx::*;

pub mod convert;
pub mod default;
pub mod iter;
pub mod ptr;
pub mod result;
pub mod slice;

#[cfg(feature = "alloc")]
cfg_group! {
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub mod boxed;

    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub mod vec;
}
