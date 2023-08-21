//! Safely cast between numbers.
//!
//! The extension trait [`NumericCast`] adds a generic method [`numeric_cast`](NumericCast::numeric_cast) for all number types.
//! The method allows users to safely cast a number to another type without losing precision.
//!
//! If the value can not be represented by the target type,
//! the method will panic with a message which tells the value, the source type name and the target type name.
//!
//! As [`numeric_cast`](NumericCast::numeric_cast) is marked by `track_caller`, the panic location will be exactly where you call the method.
//!
//! This library optimizes for code bloat. In most use cases, numeric cast always succeeds at runtime,
//! so the panic function is split from normal control flow to reduce performance impact.
//!
//! # Examples
//!
//! ```
//! use numeric_cast::NumericCast;
//!
//! let entries: u64 = 1024;
//!
//! let capacity = entries.numeric_cast::<usize>();
//! let offset: isize = entries.numeric_cast(); // by inference
//! ```
//!
//! ```should_panic
//! use numeric_cast::NumericCast;
//!
//! let n: i32 = -1;
//! let len: usize = n.numeric_cast(); // panic here
//! ```
//!
#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate //
)]
// ---
#![cfg_attr(not(test), no_std)]

mod lossless;
pub use self::lossless::*;

mod wrapping;
pub use self::wrapping::*;

mod extending;
pub use self::extending::*;

mod truncating;
pub use self::truncating::*;

mod rounding;
pub use self::rounding::*;

#[cold]
#[track_caller]
#[inline(never)]
fn panic_failure(msg: &'static str, val: &dyn core::fmt::Display, lhs: &'static str, rhs: &'static str) -> ! {
    panic!("{}: lhs: {}, rhs: {}, val: {}", msg, lhs, rhs, val)
}
