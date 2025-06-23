//! Test library for the toml_const crate.
// #![cfg(test)]

toml_const::toml_const! {
    pub const TOML_CONST_EXAMPLE: final "../example.toml";

    // / Doc comments bla bla bla
    #[derive(PartialEq)] // additional derive attributes
    static CARGO_TOML: "Cargo.toml" {
        use "../toml_const_macros/Cargo.toml";
        "non_existent.toml";
    }
}

toml_const::toml_const_ws! {pub static TOML_CONST_EXAMPLE_WS: "./example.toml"; }

toml_const::toml_const! {
    #[derive(PartialEq)]
    const NORMALIZE_TOML: "../normalize.toml";
}

#[cfg(test)]
mod tests {
    use crate::NORMALIZE_TOML;

    #[test]
    fn test_print_nornalize() {
        let toml: toml::Table = toml::from_str(include_str!("../../normalize.toml")).unwrap();

        println!("{:#?}", toml);
        // println!("{:#?}", NORMALIZE_TOML.identical_values.map());

        for item in NORMALIZE_TOML.identical_values.map().into_iter() {
            println!("{}: {:?}", item.0, item.1);
        }
    }
}
