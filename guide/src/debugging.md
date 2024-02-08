# Debugging

## Macros

PyO3's attributes (`#[pyclass]`, `#[pymodule]`, etc.) are [procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html), which means that they rewrite the source of the annotated item. You can view the generated source with the following command, which also expands a few other things:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded > expanded.rs; rustfmt expanded.rs
```

(You might need to install [rustfmt](https://github.com/rust-lang-nursery/rustfmt) if you don't already have it.)

You can also debug classic `!`-macros by adding `-Z trace-macros`:

```bash
cargo rustc --profile=check -- -Z unstable-options --pretty=expanded -Z trace-macros > expanded.rs; rustfmt expanded.rs
```

Note that those commands require using the nightly build of rust and may occasionally have bugs. See [cargo expand](https://github.com/dtolnay/cargo-expand) for a more elaborate and stable version of those commands.

## Running with Valgrind

Valgrind is a tool to detect memory management bugs such as memory leaks.

You first need to install a debug build of Python, otherwise Valgrind won't produce usable results. In Ubuntu there's e.g. a `python3-dbg` package.

Activate an environment with the debug interpreter and recompile. If you're on Linux, use `ldd` with the name of your binary and check that you're linking e.g. `libpython3.7d.so.1.0` instead of `libpython3.7.so.1.0`.

[Download the suppressions file for CPython](https://raw.githubusercontent.com/python/cpython/master/Misc/valgrind-python.supp).

Run Valgrind with `valgrind --suppressions=valgrind-python.supp ./my-command --with-options`

## Getting a stacktrace

The best start to investigate a crash such as an segmentation fault is a backtrace. You can set `RUST_BACKTRACE=1` as an environment variable to get the stack trace on a `panic!`. Alternatively you can use a debugger such as `gdb` to explore the issue. Rust provides a wrapper, `rust-gdb`, which has pretty-printers for inspecting Rust variables. Since PyO3 uses `cdylib` for Python shared objects, it does not receive the pretty-print debug hooks in `rust-gdb` ([rust-lang/rust#96365](https://github.com/rust-lang/rust/issues/96365)). The mentioned issue contains a workaround for enabling pretty-printers in this case.

 * Link against a debug build of python as described in the previous chapter
 * Run `rust-gdb <my-binary>`
 * Set a breakpoint (`b`) on `rust_panic` if you are investigating a `panic!`
 * Enter `r` to run
 * After the crash occurred, enter `bt` or `bt full` to print the stacktrace

 Often it is helpful to run a small piece of Python code to exercise a section of Rust.

 ```console
 rust-gdb --args python -c "import my_package; my_package.sum_to_string(1, 2)"
 ```
