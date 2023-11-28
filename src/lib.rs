//!
//!
#![doc = include_str!("../README.md")]

mod codegen;
pub mod consts;
mod generator;

pub use generator::run;

#[cfg(test)]
mod tests {
    // use super::*;
}
