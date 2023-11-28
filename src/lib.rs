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
    // use Datetime;


    fn asd() {
        let x: toml::Value = toml::Value::Table(Default::default());
        match x {
            toml::Value::String(_) => todo!(),
            toml::Value::Integer(_) => todo!(),
            toml::Value::Float(_) => todo!(),
            toml::Value::Boolean(_) => todo!(),
            toml::Value::Datetime(_dt) => todo!(),
            toml::Value::Array(_) => todo!(),
            toml::Value::Table(_) => todo!(),
        }
    }
}
