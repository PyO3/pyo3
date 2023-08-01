variable::set("PYO3_VERSION", "0.19.2");
file::rename(".template/Cargo.toml", "Cargo.toml");
file::rename(".template/plugin_api/Cargo.toml", "plugin_api/Cargo.toml");
file::delete(".template");
