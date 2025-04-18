#![doc = include_str!("../README.md")]
#![no_std]

use core::ops::Deref;

// re-exports
pub use macros::*;
pub use toml::value::{Date, Datetime, Offset, Time};

/// An array referencing a `'static` slice of type `T`.
#[derive(Clone, Copy, Debug)]
pub struct Array<T: 'static>(pub &'static [T]);

/// An empty value. Empty toml arrays contain this type.
#[derive(Clone, Copy, Debug)]
pub struct Empty;

impl<T: 'static + Copy> Deref for crate::Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
