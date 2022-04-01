variable::set("PYO3_VERSION", "0.16.2");
file::rename(".template/Cargo.toml", "Cargo.toml");
file::rename(".template/tox.ini", "tox.ini");
file::delete(".template");
