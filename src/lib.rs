//!
//!
#![doc = include_str!("../README.md")]

pub mod consts;
mod generator;
mod codegen;

pub use generator::run;

#[cfg(test)]
mod tests {
    // use super::*;
}
