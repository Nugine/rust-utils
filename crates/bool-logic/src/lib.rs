#![deny(clippy::all)]
#![warn(clippy::dbg_macro, clippy::todo)]

mod utils;

pub mod ast;
pub mod eval;
pub mod transform;
pub mod visit_mut;
