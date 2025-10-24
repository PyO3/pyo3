# 摘要

[介绍](index.md)

---

- [入门指南](getting-started.md)
- [在 Python 调用 Rust](rust-from-python.md)
  - [Python 模块](module.md)
  - [Python 函数](function.md)
    - [函数签名](function/signature.md)
    - [错误处理](function/error-handling.md)
  - [Python 类](class.md)
    - [类自定义](class/protocols.md)
      - [基本对象自定义](class/object.md)
      - [模拟数值类型](class/numeric.md)
      - [模拟可调用对象](class/call.md)
    - [线程安全](class/thread-safety.md)
- [在 Rust 调用 Python](python-from-rust.md)
  - [Python 对象类型](types.md)
  - [Python 异常](exception.md)
  - [调用 Python 函数](python-from-rust/function-calls.md)
  - [执行现有的 Python 代码](python-from-rust/calling-existing-code.md)
- [类型转换](conversions.md)
  - [Rust 类型到 Python 类型的映射](conversions/tables.md)
  - [转换 trait](conversions/traits.md)
- [使用 `async` 和 `await`](async-await.md)
- [并行性](parallelism.md)
- [支持自由线程的 CPython](free-threading.md)
- [调试](debugging.md)
- [功能参考](features.md)
- [性能](performance.md)
- [类型存根生成和内省](type-stub.md)
- [高级主题](advanced.md)
- [构建和分发](building-and-distribution.md)
  - [支持多个 Python 版本](building-and-distribution/multiple-python-versions.md)
- [实用的 crate](ecosystem.md)
  - [日志记录](ecosystem/logging.md)
  - [跟踪](ecosystem/tracing.md)
  - [使用 `async` 和 `await`](ecosystem/async-await.md)
- [常见问题和故障排除](faq.md)

---

[附录 A: 迁移指南](migration.md)
[附录 B: Trait 边界](trait-bounds.md)
[附录 C: Python 类型提示](python-typing-hints.md)
[变更日志](changelog.md)

---

[贡献](contributing.md)