# toml_const

<div align="center">

**TOML compile-time constants**

<!-- ![crate license](https://img.shields.io/crates/l/toml_const) -->
![no std](https://img.shields.io/badge/no__std-12a077)
[![crate](https://img.shields.io/crates/v/toml_const.svg)](https://crates.io/crates/toml_const)
[![docs](https://docs.rs/toml_const/badge.svg)](https://docs.rs/toml_const)
[![build status](https://github.com/facesthe/toml_const/actions/workflows/ci.yml/badge.svg)](https://github.com/facesthe/toml_const/actions/workflows/ci.yml)

</div>

## Getting started

```rust
use toml_const::{toml_const, toml_const_ws};

// workspace root
// ├── example.toml
// ├── toml_const       <---- you are here
// │   ├── Cargo.toml
// │   └── src
// └── toml_const_macros
//     ├── Cargo.toml
//     └── src

// include a TOML file in your project relative to your manifest directory
toml_const! {
    pub const EXAMPLE_TOML: "../example.toml";
    // multiple definitions are supported
    static CARGO_TOML: "Cargo.toml";
}

// include a file relative to your workspace root
toml_const_ws! {static EXAMPLE_TOML_WS: "example.toml";}

// table keys are capitalized struct fields
const TITLE: &str = EXAMPLE_TOML.TITLE;
assert_eq!(EXAMPLE_TOML.TITLE, EXAMPLE_TOML_WS.TITLE);
```

## Table substitution

File substitution is supported.
The first path that exists and satisfies the following conditions will be used.
These conditions are, in order of precedence:

- if a substitute path has the `use` keyword prefixed
- iif a toml file contains `use = true` at the root level

Multiple substitute files can be specified in the macro expression.
The first file containing a `use = true` key will be merged into the parent file.

These files may contain secrets or other sensitive information that you don't want to check into version control.

```rust
use toml_const::toml_const;

toml_const! {
    // example.toml is the template/parent file (must exist)
    pub static EXAMPLE_TOML: "../example.toml" {
        // if Cargo.toml exists, it will be substituted
        use "../Cargo.toml";
        // if Cargo.toml does not exist and example.toml contains
        // `use = true`, it will be substituted
        "../example.toml";
        // files that do not exist are ignored
        "non_existent.toml";
        // .. and so on
    }
}
```

## Limitations

This library does not support the full TOML specification.

It **will fail to**:

- generate arrays with distinct types (arrays containing different types, arrays of tables with different keys)
- create a struct from a table with a blank key `"" = true`

It **will modify**:

- table keys that begin with numbers
- table keys that contain invalid characters for identifiers

## TOML data types

All TOML data types are supported. Datetime related structs are re-exported from `toml`.

| data type | rust type |
| --- | --- |
| boolean | `bool` |
| integer | `i64` |
| float | `f64` |
| string | `&'static str` |
| date | `toml::value::Datetime` |
| array | `toml_const::Array<T>` |
| table | auto-generated struct |
