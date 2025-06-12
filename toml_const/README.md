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
    /// Docstring for this item
    #[derive(PartialEq, PartialOrd)] // Clone, Copy, Debug are already derived
    pub const EXAMPLE_TOML: "../example.toml";
    // multiple definitions are supported
    static CARGO_TOML: "Cargo.toml";
}

// include a file relative to your workspace root
toml_const_ws! {static EXAMPLE_TOML_WS: "example.toml";}

// table keys are capitalized struct fields
const TITLE: &str = EXAMPLE_TOML.title;
assert_eq!(EXAMPLE_TOML.title, EXAMPLE_TOML_WS.title);
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

## Normalization

A TOML file is normalized before it is generated as code. This step does not modify the original config file.

Tables within arrays will have their keys propagated across all elements. Missing keys will be filled with default values.
This means that keys can be omitted from parts of your config as long as it is defined in at least one element.

Empty arrays will be inferred to be `&'static [&'static str]`.

```toml
# this table will normalize to ...
[program]
name = "my_library"
versions = [
    { version = "0.1.0", description = "Initial release" },
    { version = "0.2.0" }, # description is omitted
    { version = "0.3.0", description = "Added support for arrays of tables", bug_fixes = [
        { issue = "1", description = "Fixed a bug with arrays of tables" },
        { issue = "2", description = "support nested arrays" },
    ] },
]

# ... this
[program]
name = "my_library"
versions = [
    { version = "0.1.0", description = "Initial release", bug_fixes = [] },
    { version = "0.2.0", description = "", bug_fixes = [] },
    { version = "0.3.0", description = "Added support for arrays of tables", bug_fixes = [
        { issue = "1", description = "Fixed a bug with arrays of tables" },
        { issue = "2", description = "support nested arrays" },
    ] },
]
```

## Unwrapping datetime

`toml::Datetime` contains fields that point to `Option`s, which need const/runtime checks.
As the toml spec defines [4 datetime formats](https://docs.rs/toml/latest/toml/value/struct.Datetime.html),
non-option types can be used to unwrap datetime values at compile time.

Datetime values are also normalized to support multiple formats defined for one key.
The union of all formats will be used to generate the final datetime format.

## Attributes

Docstrings and derive attributes are supported.
`Clone`, `Copy`, and `Debug` are automatically derived for all types.

```rust
use toml_const::toml_const;

toml_const! {
    /// # Cargo manifest file
    ///
    /// This file contains
    /// - something
    #[derive(PartialEq, Hash)]
    pub const CARGO_TOML: "Cargo.toml";
}
```

## Limitations

This library does not support the full TOML specification.

It **will fail to**:

- generate arrays with distinct types (arrays containing different types, arrays of tables with conflicting key types)
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
| date | `toml_const::Datetime` |
| array | `&'static [T]` |
| table | auto-generated struct |
