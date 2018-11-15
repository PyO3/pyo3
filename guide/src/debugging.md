# Debugging

## Macros

Pyo3's attributes, `#[pyclass]`, `#[pymodule]`, etc. are [procedural macros](https://doc.rust-lang.org/unstable-book/language-features/proc-macro.html), which means that rewrite the source of the annotated item. You can view the generated source with the following command, which also expands a few other things:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded > expanded.rs; rustfmt expanded.rs
```

(You might need to install [rustfmt](https://github.com/rust-lang-nursery/rustfmt) if you don't already have it.)

You can also debug classic `!`-macros by adding -Z trace-macros`:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded -Z trace-macros > expanded.rs; rustfmt expanded.rs
```

See [cargo expand](https://github.com/dtolnay/cargo-expand) for a more elaborate version of those commands.

## Linking

When building, you can set `PYTHON_SYS_EXECUTABLE` to the python interpreter that pyo3 should be linked to. You might need to set the `python2` or `python3` feature accordingly. On linux/mac you might have to change `LD_LIBRARY_PATH` to include libpython, while on windows you might need to set `LIB` to include `pythonxy.lib` (where x and y are major and minor version), which is normally either in the `libs` or `Lib` folder of a python installation. Also make sure that python is in `PATH` when you're not using  `PYTHON_SYS_EXECUTABLE`.
