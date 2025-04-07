//!
//!
#![doc = include_str!("../README.md")]

mod codegen;
pub mod consts;
mod generator;

pub use generator::run;

pub use macros::*;
// re-exports
pub use toml::value::{Date, Datetime, Offset, Time};

/// Const array
pub struct Array<T: 'static>(&'static [T]);

/// An empty value. Empty toml arrays contain this type.
#[derive(Clone, Copy)]
pub struct Empty;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as toml_const;

    macros::test!("./lib.rs");

    macros::toml_const! {pub TOML_CONST_ITEM: "./example.toml" }
    // const X: usize = TOML_CONST_ITEM.DATABASE.CREDENTIALS.ODT2;
    // macros::toml_const!("./Cargo.toml");

    #[test]
    fn test_print_time_secs() {
        println!("unix time secs: {}", TIME);
        let x = Array::<usize>(&[1]);
    }

    macro_rules! asd {
        ($some_id: ident) => {
            const $some_id: usize = 0;
        };
    }

    asd!(ASD);
}
