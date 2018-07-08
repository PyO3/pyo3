# Debugging

Pyo3's attributes, `#[pyclass]`, `#[pymodinit]`, etc. are [procedural macros](https://doc.rust-lang.org/unstable-book/language-features/proc-macro.html), which means that rewrite the source of the annotated item. You can view the generated source with the following command, which also expands a few other things:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded > expanded.rs; rustfmt expanded.rs
```

(You might need to install [rustfmt](https://github.com/rust-lang-nursery/rustfmt) if you don't already have it.)

You can also debug classic `!`-macros by adding -Z trace-macros`:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded -Z trace-macros > expanded.rs; rustfmt expanded.rs
```

See [cargo expand](https://github.com/dtolnay/cargo-expand) for a more elaborate version of those commands.
