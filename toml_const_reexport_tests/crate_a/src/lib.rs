pub use toml_const;

#[macro_export]
macro_rules! toml_config {
    ($($input:tt)*) => {
        $crate::toml_const::toml_const! { $($input)* }
    };
}
