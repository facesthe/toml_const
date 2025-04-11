# toml_const

<div align="center">

**TOML compile-time constants**

<!-- ![crate license](https://img.shields.io/crates/l/toml_const) -->
![no std](https://img.shields.io/badge/no__std-12a077)
![crate version](https://img.shields.io/crates/v/toml_const)
![docs](https://img.shields.io/docsrs/toml_const)
![build status](https://img.shields.io/github/actions/workflow/status/facesthe/toml_const/.github%2Fworkflows%2Fci.yml)

</div>

## Getting started

```rust
use toml_const::{toml_const, toml_const_ws};

// public struct
// include a TOML file in your project relative to your manifest directory
toml_const! {
    pub const EXAMPLE_TOML: "example.toml"
    // multiple definitions are supported
    static CARGO_TOML: "Cargo.toml"
}

// private struct
// include a file relative to your workspace root
toml_const_ws! {static EXAMPLE_TOML_WS: "example.toml"}

// table keys are capitalized struct fields
const TITLE: &str = EXAMPLE_TOML.TITLE;
assert_eq!(EXAMPLE_TOML.TITLE, EXAMPLE_TOML_WS.TITLE);
```

## Table substitution

Multiple child files can be specified in the macro expression.
The first file containing a `use = true` key will be merged into the parent file.

These files may contain secrets or other sensitive information that you don't want to check into version control.

```rust
use toml_const::toml_const;

toml_const! {
    // example.toml is the template/parent file (must exist)
    pub static EXAMPLE_TOML: "example.toml" {
        // the first file with use = true will be merged into the parent file
        //
        // as none of the files have use = true,
        // only example.toml is used
        "Cargo.toml";
        "example.toml";
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
