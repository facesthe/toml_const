#![doc = include_str!("../README.md")]

mod codegen;
pub mod consts;
mod generator;

use std::ops::Deref;

pub use generator::run;

// re-exports
pub use macros::*;
pub use toml::value::{Date, Datetime, Offset, Time};

use crate as toml_const;
macros::toml_const_ws!(pub TOML_CONST_EXAMPLE_WS: "./example.toml");

/// Const array
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
    macros::toml_const! {pub TOML_CONST_EXAMPLE: "./example.toml"}
    const _: TomlConstExample = TOML_CONST_EXAMPLE;

    #[test]
    fn test_asd() {
        println!("{:?}", TOML_CONST_EXAMPLE);
        println!(
            "size of TOML_CONST_EXAMPLE: {}",
            std::mem::size_of::<TomlConstExample>()
        );
    }
}
