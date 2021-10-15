//!
//! This crate provides a macro `qas`, which can convert C into Rust.
//!
//! Example:
//!
//! lib:
//! ```rust
//! use qas::prelude::*;
//!
//! qas!("tests/c/arithmetic.c");
//!
//! fn main() {
//!     assert_eq!(add(7, 32), 39)
//! }
//! ```

#![no_std]

extern crate qas_macro;

mod traits;
pub mod builtin;

pub mod prelude {
    use super::*;

    pub use qas_macro::qas;
    pub use traits::*;
}
