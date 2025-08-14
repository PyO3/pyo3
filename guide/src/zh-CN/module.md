# Python 模块

你可以使用 `#[pymodule]` 创建一个模块：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

/// 这个模块是用 Rust 实现的。
#[pymodule]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)
}
```

`#[pymodule]` 过程宏负责将模块的初始化函数导出到 Python。

模块的名称默认为 Rust 函数的名称。你可以使用 `#[pyo3(name = "custom_name")]` 来覆盖模块名称：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule(name = "custom_name")]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)
}
```

模块的名称必须与 `.so` 或 `.pyd` 文件的名称匹配。否则，在 Python 中导入时会收到导入错误，消息为：`ImportError: dynamic module does not define module export function (PyInit_name_of_your_module)` 即 `ImportError: 动态模块未定义模块导出函数 (PyInit_你的模块名称)`

要导入模块，可以：
 - 如[手动构建](building-and-distribution.md)中所述复制共享库，或者
 - 使用工具，例如使用 [maturin](https://github.com/PyO3/maturin) 的 `maturin develop` 或使用 [setuptools-rust](https://github.com/PyO3/setuptools-rust) 的 `python setup.py develop`。

## 文档

模块初始化函数的 [Rust 文档注释](https://doc.rust-lang.org/stable/book/ch03-04-comments.html) 将自动应用于模块的 Python 文档字符串。

例如，基于上面的代码，这将打印 `这个模块是用 Rust 实现的。`：

```python
import my_extension

print(my_extension.__doc__)
```

## Python 子模块

你可以使用 [`Bound<'_, PyModule>::add_submodule()`]({{#PYO3_DOCS_URL}}/pyo3/prelude/trait.PyModuleMethods.html#tymethod.add_submodule) 在单个扩展模块中创建模块层次结构。
例如，你可以定义模块 `parent_module` 和 `parent_module.child_module`。

```rust
use pyo3::prelude::*;

#[pymodule]
fn parent_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_child_module(m)?;
    Ok(())
}

fn register_child_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let child_module = PyModule::new(parent_module.py(), "child_module")?;
    child_module.add_function(wrap_pyfunction!(func, &child_module)?)?;
    parent_module.add_submodule(&child_module)
}

#[pyfunction]
fn func() -> String {
    "func".to_string()
}

# Python::attach(|py| {
#    use pyo3::wrap_pymodule;
#    use pyo3::types::IntoPyDict;
#    use pyo3::ffi::c_str;
#    let parent_module = wrap_pymodule!(parent_module)(py);
#    let ctx = [("parent_module", parent_module)].into_py_dict(py).unwrap();
#
#    py.run(c_str!("assert parent_module.child_module.func() == 'func'"), None, Some(&ctx)).unwrap();
# })
```

请注意，这并不定义一个包，因此不会允许 Python 代码通过 `from parent_module import child_module` 直接导入子模块。更多信息，请参阅 [#759](https://github.com/PyO3/pyo3/issues/759) 和 [#1517](https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021)。

对于嵌套模块，不需要在它们上添加 `#[pymodule]`，这仅在顶级模块上是必需的。

## 声明式模块

另一种基于 Rust 内联模块的语法也可用于声明模块。

例如：
```rust
# mod declarative_module_test {
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
mod my_extension {
    use super::*;

    #[pymodule_export]
    use super::double; // 将 double 函数作为模块的一部分导出

    #[pymodule_export]
    const PI: f64 = std::f64::consts::PI; // 将 PI 常量作为模块的一部分导出

    #[pyfunction] // 这将成为模块的一部分
    fn triple(x: usize) -> usize {
        x * 3
    }

    #[pyclass] // 这将成为模块的一部分
    struct Unit;

    #[pymodule]
    mod submodule {
        // 这是一个子模块
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // 模块初始化时运行的任意代码
        m.add("double2", m.getattr("double")?)
    }
}
# }
```

`#[pymodule]` 宏会自动将其中声明的 `#[pyclass]` 宏的 `module` 属性设置为其名称。
对于嵌套模块，会自动添加父模块的名称。
在以下示例中，`Unit` 类的 `module` 将为 `my_extension.submodule`，因为它是正确嵌套的，
但 `Ext` 类的 `module` 将为默认的 `builtins`，因为它未嵌套。

```rust
# mod declarative_module_module_attr_test {
use pyo3::prelude::*;

#[pyclass]
struct Ext;

#[pymodule]
mod my_extension {
    use super::*;

    #[pymodule_export]
    use super::Ext;

    #[pymodule]
    mod submodule {
        use super::*;
        // 这是一个子模块

        #[pyclass] // 这将成为模块的一部分
        struct Unit;
    }
}
# }
```
可以使用 `#[pyo3(module = "MY_MODULE")]` 选项自定义 `pymodule()` 的 `module` 值。

对于非顶级模块，可以向 `pymodule()` 提供 `submodule` 参数——对于嵌套在 `#[pymodule]` 中的模块，它会自动设置。