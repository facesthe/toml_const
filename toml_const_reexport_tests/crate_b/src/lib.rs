toml_const_reexport_test_a::toml_config! {
    pub const CONFIG: "config.toml";
}

#[cfg(test)]
mod tests {
    use super::CONFIG;

    #[test]
    fn reexported_macro_needs_no_direct_toml_const_dependency() {
        assert_eq!(CONFIG.name, "test");
        assert_eq!(CONFIG.date.date.year, 2024);
        assert_eq!(CONFIG.servers.map().get("alpha").unwrap().port, 8080);
    }
}
