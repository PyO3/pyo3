variable::set("PYO3_VERSION", "0.29.0");
file::rename(".template/Cargo.toml", "Cargo.toml");
file::rename(".template/pyproject.toml", "pyproject.toml");
file::delete(".template");
