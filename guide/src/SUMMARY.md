# Summary

[Introduction](index.md)

---

- [Getting started](getting-started.md)
- [Using Rust from Python](rust-from-python.md)
  - [Python modules](module.md)
  - [Python functions](function.md)
    - [Function signatures](function/signature.md)
    - [Error handling](function/error-handling.md)
  - [Python classes](class.md)
    - [Class customizations](class/protocols.md)
      - [Basic object customization](class/object.md)
      - [Emulating numeric types](class/numeric.md)
      - [Emulating callable objects](class/call.md)
    - [Thread safety](class/thread-safety.md)
- [Calling Python from Rust](python-from-rust.md)
  - [Python object types](types.md)
  - [Python exceptions](exception.md)
  - [Calling Python functions](python-from-rust/function-calls.md)
  - [Executing existing Python code](python-from-rust/calling-existing-code.md)
- [Type conversions](conversions.md)
  - [Mapping of Rust types to Python types](conversions/tables.md)
  - [Conversion traits](conversions/traits.md)
- [Using `async` and `await`](async-await.md)
- [Parallelism](parallelism.md)
- [Supporting Free-Threaded Python](free-threading.md)
- [Debugging](debugging.md)
- [Features reference](features.md)
- [Performance](performance.md)
- [Type stub generation and introspection](type-stub.md)
- [Advanced topics](advanced.md)
- [Building and distribution](building-and-distribution.md)
  - [Supporting multiple Python versions](building-and-distribution/multiple-python-versions.md)
- [Useful crates](ecosystem.md)
  - [Logging](ecosystem/logging.md)
  - [Tracing](ecosystem/tracing.md)
  - [Using `async` and `await`](ecosystem/async-await.md)
- [FAQ and troubleshooting](faq.md)

---

[Appendix A: Migration guide](migration.md)
[Appendix B: Trait bounds](trait-bounds.md)
[Appendix C: Python typing hints](python-typing-hints.md)
[CHANGELOG](changelog.md)

---

[Contributing](contributing.md)
