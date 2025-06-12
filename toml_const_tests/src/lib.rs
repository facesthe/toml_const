//! Test library for the toml_const crate.
// #![cfg(test)]

toml_const::toml_const! {
    pub const TOML_CONST_EXAMPLE: final "../example.toml";

    /// Doc comments bla bla bla
    #[derive(PartialEq, Hash)] // additional derive attributes
    static CARGO_TOML: "Cargo.toml" {
        use "src/toml_const_macros/Cargo.toml";
        "non_existent.toml";
    }
}

toml_const::toml_const_ws! {pub static TOML_CONST_EXAMPLE_WS: "./example.toml"; }

toml_const::toml_const_ws! {
    pub static TOML_CONST_EXAMPLE_WS_UNWRAP: "./example.toml";
}

toml_const::toml_const! {
    const NORMALIZE_TOML: "../normalize.toml";
}

#[test]
fn test_ws_crate_macro_equal() {
    assert_eq!(TOML_CONST_EXAMPLE.age, TOML_CONST_EXAMPLE_WS_UNWRAP.age);
    assert_eq!(
        TOML_CONST_EXAMPLE.colors,
        TOML_CONST_EXAMPLE_WS_UNWRAP.colors
    );
}
