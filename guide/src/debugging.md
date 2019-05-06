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

See [cargo expand](https://github.com/dtolnay/cargo-expand) for a more elaborate version of those commands.

## Running with Valgrind

Valgrind is a tool to detect memory management bugs such as memory leaks.

You first need to install a debug build of Python, otherwise Valgrind won't produce usable results. In Ubuntu there's e.g. a `python3-dbg` package.

Activate an environment with the debug interpreter and recompile. If you're on Linux, use `ldd` with the name of your binary and check that you're linking e.g. `libpython3.6dm.so.1.0` instead of `libpython3.6m.so.1.0`.

[Download the suppressions file for cpython](https://raw.githubusercontent.com/python/cpython/master/Misc/valgrind-python.supp).

Run Valgrind with `valgrind --suppressions=valgrind-python.supp ./my-command --with-options`

## Getting a stacktrace

The best start to investigate a crash such as an segmentation fault is a backtrace.

 * Link against a debug build of python as described in the previous chapter
 * Run `gdb <my-binary>`
 * Enter `r` to run
 * After the crash occurred, enter `bt` or `bt full` to print the stacktrace
