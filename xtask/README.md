## Commands to test PyO3.

To run these commands, you should be in PyO3's root directory, and run (for example) `cargo xtask all`.

```
USAGE:
    xtask.exe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    ci          Runs everything
    clippy      Runs `clippy`, denying all warnings
    coverage    Runs `cargo llvm-cov` for the PyO3 codebase
    default     Only runs the fast things (this is used if no command is specified)
    doc         Attempts to render the documentation
    fmt         Checks Rust and Python code formatting with `rustfmt` and `black`
    help        Prints this message or the help of the given subcommand(s)
    test        Runs various variations on `cargo test`
    test-py     Runs the tests in examples/ and pytests/
```