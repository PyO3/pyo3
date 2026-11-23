variable::set("PYO3_VERSION", "0.29.3");
file::rename(".template/Cargo.toml", "Cargo.toml");
file::rename(".template/pyproject.toml", "pyproject.toml");
file::rename(".template/meson.build", "meson.build");
file::delete(".template");
file::delete("_build_backend.py");
