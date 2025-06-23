//! Test library for the toml_const crate.
// #![cfg(test)]

// toml_const::toml_const! {
//     pub const TOML_CONST_EXAMPLE: final "../example.toml";

//     /// Doc comments bla bla bla
//     #[derive(PartialEq, Hash)] // additional derive attributes
//     static CARGO_TOML: "Cargo.toml" {
//         use "../toml_const_macros/Cargo.toml";
//         "non_existent.toml";
//     }
// }

// toml_const::toml_const_ws! {pub static TOML_CONST_EXAMPLE_WS: "./example.toml"; }

toml_const::toml_const! {
    // #[derive(PartialEq, PartialOrd)]
    const NORMALIZE_TOML: "../normalize.toml";
}

fn asd() {
    let x = NORMALIZE_TOML.tables.b.map();
    let b = NORMALIZE_TOML.tables.b;
}

// #[cfg(test)]
// mod tests {
//     use crate::NORMALIZE_TOML;

//     #[test]
//     fn test_print_mornalize() {
//         let toml: toml::Table = toml::from_str(include_str!("../../normalize.toml")).unwrap();

//         println!("{:#?}", toml);
//         println!("{:#?}", NORMALIZE_TOML.identical_values);
//     }
// }
