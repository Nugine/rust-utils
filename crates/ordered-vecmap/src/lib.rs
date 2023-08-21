#![deny(clippy::all, clippy::missing_inline_in_public_items, clippy::must_use_candidate)]
// ---
#![no_std]

extern crate alloc;

pub mod vecmap;
pub mod vecset;

pub use self::vecmap::VecMap;
pub use self::vecset::VecSet;
