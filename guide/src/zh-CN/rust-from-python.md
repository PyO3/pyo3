# 在 Python 调用 Rust

指南的本章旨在解释如何将 Rust 代码包装成 Python 对象。

PyO3 使用 Rust 的“过程宏”来提供一个强大而简单的 API，用于指定哪些 Rust 代码应该映射到 Python 对象。

PyO3 可以创建三种类型的 Python 对象：

- Python 模块，通过 `#[pymodule]` 宏
- Python 函数，通过 `#[pyfunction]` 宏
- Python 类，通过 `#[pyclass]` 宏（加上 `#[pymethods]` 来定义这些类的方法）

下面的子章节将依次介绍这些内容。