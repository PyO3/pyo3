# Debugging

Pyo3's attributes, `#[class]`, `#[modinit]`, etc. are [procedural macros](https://doc.rust-lang.org/unstable-book/language-features/proc-macro.html), which means that rewrite the source of the annotated item. You can view the generated source with the following command, which also expands a few other things:

```rust
cargo rustc -- -Z unstable-options --pretty=expanded > expanded.rs; rustfmt expanded.rs
```

(You might need to install [rustfmt](https://github.com/rust-lang-nursery/rustfmt) if you don't already have it.)

You can also debug classic `!`-macros by adding -Z trace-macros`:

```rust
cargo rustc -- -Z unstable-options --pretty=expanded -Z trace-macros > expanded.rs; rustfmt expanded.rs
```
