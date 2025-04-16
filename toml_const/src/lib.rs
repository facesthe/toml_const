#![doc = include_str!("../../README.md")]
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

#[cfg(test)]
mod tests {
    use crate as toml_const;

    // example.toml must parse completely
    macros::toml_const! {
        pub const TOML_CONST_EXAMPLE: "../example.toml";

        static CARGO_TOML: "Cargo.toml" {
            "src/toml_const_macros/Cargo.toml";
            "non_existent.toml";
        }
    }

    macros::toml_const_ws! {pub static TOML_CONST_EXAMPLE_WS: "./example.toml"; }
}
