#![allow(nonstandard_style)]
#![allow(overflowing_literals)]

#[macro_use]
mod macros;

// 10.0.22000.0
pub mod km;
pub mod shared;

mod ext;
mod result;

pub use self::{
    ext::{OkExt, UnicodeStringExt},
    result::Result,
};
