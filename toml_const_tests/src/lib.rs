//! Test library for the toml_const crate.
#![cfg(test)]

toml_const::toml_const! {
    pub const TOML_CONST_EXAMPLE: "../example.toml";

    static CARGO_TOML: "Cargo.toml" {
        use "src/toml_const_macros/Cargo.toml";
        "non_existent.toml";
    }
}

toml_const::toml_const_ws! {pub static TOML_CONST_EXAMPLE_WS: "./example.toml"; }

#[test]
fn test_ws_crate_macro_equal() {
    assert_eq!(TOML_CONST_EXAMPLE.AGE, TOML_CONST_EXAMPLE_WS.AGE);
    assert_eq!(TOML_CONST_EXAMPLE.COLORS.0, TOML_CONST_EXAMPLE_WS.COLORS.0);
}
