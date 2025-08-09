# 安装

要开始使用PyO3，您需要三样东西：Rust工具链、Python环境以及构建方式。我们将在下面逐一介绍这些内容。

> 如果您想与PyO3维护者和其他PyO3用户交流，请考虑加入[PyO3 Discord服务器](https://discord.gg/33kcChzH7f)。我们很想了解您的入门体验，这样我们就可以让PyO3对每个人都尽可能易于访问！

## Rust

首先，确保您的系统上已安装Rust。如果您还没有安装，请尝试按照[这里](https://www.rust-lang.org/tools/install)的说明进行安装。PyO3在`stable`和`nightly`版本上都可以运行，所以您可以选择最适合您的版本。最低要求的Rust版本是1.74。

如果您可以运行`rustc --version`并且版本足够新，那就可以开始了！

## Python

要使用PyO3，您至少需要Python 3.7。虽然您可以简单地使用系统上的默认Python解释器，但建议使用虚拟环境。

## 虚拟环境

虽然您可以使用任何您喜欢的虚拟环境管理器，但我们特别推荐使用`pyenv`，特别是如果您想要为多个不同的Python版本进行开发或测试，这就是本指南中示例将使用的工具。`pyenv`的安装说明可以在[这里](https://github.com/pyenv/pyenv#a-getting-pyenv)找到。（注意：要获得`pyenv activate`和`pyenv virtualenv`命令，您还需要安装[`pyenv-virtualenv`](https://github.com/pyenv/pyenv-virtualenv)插件。[pyenv安装程序](https://github.com/pyenv/pyenv-installer#installation--update--uninstallation)将一起安装这两个工具。）

保留使用`pyenv`安装时使用的源代码可能很有用，以便将来调试时可以查看原始源文件。这可以通过在`pyenv install`命令中传递`--keep`标志来完成。

例如：

```bash
pyenv install 3.12 --keep
```

### 构建

有许多构建和Python包管理系统，比如[`setuptools-rust`](https://github.com/PyO3/setuptools-rust)或[手动](./building-and-distribution.md#manual-builds)构建。我们推荐使用`maturin`，您可以在[这里](https://maturin.rs/installation.html)安装它。它是专门为与PyO3配合使用而开发的，提供了最"开箱即用"的体验，特别是如果您的目标是发布到PyPI。`maturin`只是一个Python包，所以您可以用安装其他Python包的相同方式来添加它。

系统Python：
```bash
pip install maturin --user
```

pipx：
```bash
pipx install maturin
```

pyenv：
```bash
pyenv activate pyo3
pip install maturin
```

poetry：
```bash
poetry add -G dev maturin
```

安装后，您可以运行`maturin --version`来检查是否正确安装了它。

# 创建新项目

首先，您应该创建将包含新项目的文件夹和虚拟环境。这里我们将使用推荐的`pyenv`：

```bash
mkdir pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```

之后，您应该安装构建管理器。在这个示例中，我们将使用`maturin`。激活虚拟环境后，将`maturin`添加到其中：

```bash
pip install maturin
```

现在您可以初始化新项目：

```bash
maturin init
```

如果`maturin`已经安装，您也可以直接使用它来创建新项目：

```bash
maturin new -b pyo3 pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```

# 添加到现有项目

遗憾的是，`maturin`目前无法在现有项目中运行，所以如果您想在现有项目中使用Python，基本上有两个选择：

1. 如上所述创建一个新项目，并将现有代码移动到该项目中
2. 根据需要手动编辑项目配置
> ℹ️ **译者注：maturin 在现有项目中存在限制的原因**  
> - **配置冲突**：现有 `Cargo.toml` 可能缺少必要设置（如 `crate-type = ["cdylib"]`）或存在与 Python 绑定冲突的配置（如 `bin` 类型）  
> - **设计目标**：`maturin` 专注于快速创建独立混合项目（预置 Rust + Python 绑定结构），而非改造现有代码

如果您选择第二个选项，以下是您需要注意的事项：

## Cargo.toml

确保您想要从Python访问的Rust crate被编译为库。您也可以有二进制输出，但您想要从Python访问的代码必须在库部分。此外，确保crate类型是`cdylib`，并如下添加PyO3作为依赖项：

```toml
# 如果您在`Cargo.toml`中已经有[package]信息，可以忽略此部分！
[package]
# 这里的`name`是包的名称。
name = "pyo3_start"
# 这些是良好的默认值：
version = "0.1.0"
edition = "2021"

[lib]
# 原生库的名称。这是在Python中导入库时将使用的名称（如`import string_sum`）。
# 如果您更改了这个，还必须更改`src/lib.rs`中`#[pymodule]`的名称。
name = "pyo3_example"

# "cdylib"是生成Python可以导入的共享库所必需的。
crate-type = ["cdylib"]

[dependencies]
pyo3 = { {{#PYO3_CRATE_VERSION}}, features = ["extension-module"] }
```

## pyproject.toml

您还应该创建一个包含以下内容的`pyproject.toml`：

```toml
[build-system]
requires = ["maturin>=1,<2"]
build-backend = "maturin"

[project]
name = "pyo3_example"
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
```

## 运行代码

之后，您可以设置Rust代码在Python中可用，如下所示；例如，您可以将此代码放在`src/lib.rs`中：

```rust,no_run
/// 用Rust实现的Python模块。此函数的名称必须与
/// `Cargo.toml`中的`lib.name`设置匹配，否则Python将无法导入模块。
#[pyo3::pymodule]
mod pyo3_example {
    use pyo3::prelude::*;

    /// 将两个数字的和格式化为字符串。
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}
```

现在您可以运行`maturin develop`来准备Python包，之后您可以像这样使用它：

```bash
$ maturin develop
# 当maturin运行编译时会有大量进度输出...
$ python
>>> import pyo3_example
>>> pyo3_example.sum_as_string(5, 20)
'25'
```

有关如何在Rust中使用Python代码的更多说明，请参见[Python from Rust](python-from-rust.md)页面。

## Maturin导入钩子

在开发过程中，代码中的任何更改都需要在测试之前运行`maturin develop`。为了简化开发过程，您可能想要安装[Maturin Import Hook](https://github.com/PyO3/maturin-import-hook)，它会在导入有代码更改的库时自动运行`maturin develop`。
