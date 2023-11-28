//!
//!
#![doc = include_str!("../README.md")]

pub mod cli;
pub mod consts;
mod generator;
mod package_navi;

pub use generator::run;

#[cfg(test)]
mod tests {
    // use super::*;
}
