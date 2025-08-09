# PyO3 用户指南

欢迎阅读 PyO3 用户指南！本书是 [PyO3 API 文档](https://docs.rs/pyo3) 的补充说明。它通过详实的示例与解析，系统阐述 PyO3 的全场景应用方案。

本指南按以下逻辑顺序展开：
  1. 入门指南
  2. 封装 Rust 代码供Python调用
  3. 在 Rust 中调用 Python 代码
  4. 高级概念详解

请从左侧章节中选择跳转到个别主题，或继续下方以开始 PyO3 的 README。

<hr style="opacity:0.2">

````markdown
# PyO3

[![actions status](https://img.shields.io/github/actions/workflow/status/PyO3/pyo3/ci.yml?branch=main&logo=github&style=)](https://github.com/PyO3/pyo3/actions)
[![benchmark](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/PyO3/pyo3)
[![codecov](https://img.shields.io/codecov/c/gh/PyO3/pyo3?logo=codecov)](https://codecov.io/gh/PyO3/pyo3)
[![crates.io](https://img.shields.io/crates/v/pyo3?logo=rust)](https://crates.io/crates/pyo3)
[![minimum rustc 1.63](https://img.shields.io/badge/rustc-1.63+-blue?logo=rust)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![discord server](https://img.shields.io/discord/1209263839632424990?logo=discord)](https://discord.gg/33kcChzH7f)
[![contributing notes](https://img.shields.io/badge/contribute-on%20github-Green?logo=github)](https://github.com/PyO3/pyo3/blob/main/Contributing.md)

PyO3 是 [Rust](https://www.rust-lang.org/) 与 [Python](https://www.python.org/) 的绑定库，提供创建原生 Python 扩展模块的工具，并支持在 Rust 二进制程序中运行和交互 Python 代码。

- 用户指南：[stable](https://pyo3.rs) | [main](https://pyo3.rs/main)

- API 文档：[stable](https://docs.rs/pyo3/) | [main](https://pyo3.rs/main/doc)

## 使用

需要 Rust 1.74 或更高版本。

PyO3 支持以下 Python 版本：
  - CPython 3.7 或更高版本
  - PyPy 7.3（Python 3.9+）
  - GraalPy 24.2 或更高版本（Python 3.11+）

您可以使用 PyO3 在 Rust 中编写原生 Python 模块，或者在 Rust 二进制文件中嵌入 Python。以下部分依次解释这些内容。

### 从 Python 使用 Rust

PyO3 可用于生成原生 Python 模块。首次尝试推荐使用 [`maturin`](https://github.com/PyO3/maturin)。`maturin` 通过最小化配置实现基于 Rust 的 Python 包的构建与发布。以下步骤安装 `maturin`，使用它生成并构建一个新的 Python 包，然后启动 Python 来导入并执行包中的函数。

首先，按照以下命令创建一个新目录，其中包含一个新的 Python `virtualenv`，并使用 Python 的包管理器 `pip` 将 `maturin` 安装到 virtualenv 中：

```bash
# (将 `string_sum` 替换为所需的包名。)
$ mkdir string_sum
$ cd string_sum
$ python -m venv .env
$ source .env/bin/activate
$ pip install maturin
```

仍然在这个 `string_sum` 目录中，现在运行 `maturin init`。这将生成新的包源代码。在选择要使用的绑定时，选择 pyo3 绑定：

```bash
$ maturin init
✔ 🤷 What kind of bindings to use? · pyo3
  ✨ Done! New project created string_sum
```

此命令生成的最重要文件是 `Cargo.toml` 和 `lib.rs`，大致如下所示：

**`Cargo.toml`**

```toml
[package]
name = "string_sum"
version = "0.1.0"
edition = "2021"

[lib]
# 原生库的名称。这是 Python 中用于导入库的名称
# （即 `import string_sum`）。如果您更改此名称，您还必须更改
# `src/lib.rs` 中 `#[pymodule]` 的名称。
name = "string_sum"
# "cdylib" 是生成供 Python 导入的共享库所必需的。
#
# 下游 Rust 代码（包括 `bin/`、`examples/` 和 `tests/` 中的代码）将无法
# `use string_sum;` 除非包括 "rlib" 或 "lib" crate 类型，例如：
# crate-type = ["cdylib", "rlib"]
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.25.1", features = ["extension-module"] }
```

**`src/lib.rs`**

```rust
use pyo3::prelude::*;

/// 将两个数字的和格式化为字符串。
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// 一个用 Rust 实现的 Python 模块。此函数的名称必须与
///  `Cargo.toml` 中的 `lib.name` 设置匹配，
/// 否则 Python 将无法导入该模块。
#[pymodule]
fn string_sum(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
```

最后，运行 `maturin develop`。这将构建包并将其安装到先前创建并激活的 Python virtualenv 中。然后，该包即可在 `python` 使用：

```bash
$ maturin develop
# lots of progress output as maturin runs the compilation...
$ python
>>> import string_sum
>>> string_sum.sum_as_string(5, 20)
'25'
```

要对包进行更改，只需编辑 Rust 源代码，然后重新运行 `maturin develop` 以重新编译。

要将其作为单个复制粘贴运行，请使用下面的 bash 脚本（用所需的包名称替换第一个命令中的 `string_sum`）：

```bash
mkdir string_sum && cd "$_"
python -m venv .env
source .env/bin/activate
pip install maturin
maturin init --bindings pyo3
maturin develop
```

如果您想能够运行 `cargo test` 或在 Cargo 工作区中使用此项目，并遇到链接器问题，请参阅 [FAQ](https://pyo3.rs/latest/faq.html#i-cant-run-cargo-test-or-i-cant-build-in-a-cargo-workspace-im-having-linker-issues-like-symbol-not-found-or-undefined-reference-to-_pyexc_systemerror) 中的一些解决方法。

除了使用 `maturin`，还可以使用 [`setuptools-rust`](https://github.com/PyO3/setuptools-rust) 或 [手动](https://pyo3.rs/latest/building-and-distribution.html#manual-builds) 构建。两者比 `maturin` 提供更多灵活性，但需要更多配置才能入门。

### 从 Rust 使用 Python

要将 Python 嵌入到 Rust 二进制文件中，您需要确保 Python 安装包含共享库。以下步骤演示如何确保这一点（针对 Ubuntu），然后给出一些示例代码，用于运行嵌入式 Python 解释器。

要在 Ubuntu 上安装 Python 共享库：

```bash
sudo apt install python3-dev
```

要在基于 RPM 的发行版（例如 Fedora、Red Hat、SuSE）上安装 Python 共享库，请安装 `python3-devel` 包。

使用 `cargo new` 启动一个新项目，并将 `pyo3` 添加到 `Cargo.toml` 中，如下所示：

```toml
[dependencies.pyo3]
version = "0.25.1"
features = ["auto-initialize"]
```

示例程序显示 `sys.version` 的值和当前用户名：

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::ffi::c_str;

fn main() -> PyResult<()> {
    Python::attach(|py| {
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;

        let locals = [("os", py.import("os")?)].into_py_dict(py)?;
        let code = c_str!("os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'");
        let user: String = py.eval(code, None, Some(&locals))?.extract()?;

        println!("Hello {}, I'm Python {}", user, version);
        Ok(())
    })
}
```

指南有一个[部分](https://pyo3.rs/latest/python-from-rust.html)，包含大量关于此主题的示例。

## 工具和库

- [maturin](https://github.com/PyO3/maturin) _构建和发布带有 pyo3、rust-cpython 或 cffi 绑定的 crate，以及作为 Python 包的 Rust 二进制文件_
- [setuptools-rust](https://github.com/PyO3/setuptools-rust) _用于 Rust 支持的 Setuptools 插件_。
- [pyo3-built](https://github.com/PyO3/pyo3-built) _简单的宏，用于将使用 [`built`](https://crates.io/crates/built) crate 获取的元数据公开为 [`PyDict`](https://docs.rs/pyo3/*/pyo3/types/struct.PyDict.html)_
- [rust-numpy](https://github.com/PyO3/rust-numpy) _NumPy C-API 的 Rust 绑定_
- [dict-derive](https://github.com/gperinazzo/dict-derive) _派生 FromPyObject 以自动将 Python 字典转换为 Rust 结构体_
- [pyo3-log](https://github.com/vorner/pyo3-log) _从 Rust 到 Python 日志的桥梁_
- [pythonize](https://github.com/davidhewitt/pythonize) _用于将 Rust 对象转换为 JSON 兼容 Python 对象的 Serde 序列化器_
- [pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes) _用于与 Python 的 Asyncio 库和 Rust 的异步运行时进行互操作的实用工具。_
- [rustimport](https://github.com/mityax/rustimport) _直接从 Python 导入 Rust 文件或 crate，无需手动编译步骤。默认提供 pyo3 集成，并自动生成 pyo3 绑定代码。_
- [pyo3-arrow](https://crates.io/crates/pyo3-arrow) _用于 pyo3 的轻量级 [Apache Arrow](https://arrow.apache.org/) 集成。_
- [pyo3-bytes](https://crates.io/crates/pyo3-bytes) _[`bytes`](https://crates.io/crates/bytes) 和 pyo3 之间的集成。_
- [pyo3-object_store](https://github.com/developmentseed/obstore/tree/main/pyo3-object_store) _[`object_store`](https://docs.rs/object_store) 和 [`pyo3`](https://github.com/PyO3/pyo3) 之间的集成。_

## 示例

- [arro3](https://github.com/kylebarron/arro3) _Apache Arrow 的最小 Python 库，连接到 Rust arrow crate。_
    - [arro3-compute](https://github.com/kylebarron/arro3/tree/main/arro3-compute) _`arro3-compute`_
    - [arro3-core](https://github.com/kylebarron/arro3/tree/main/arro3-core) _`arro3-core`_
    - [arro3-io](https://github.com/kylebarron/arro3/tree/main/arro3-io) _`arro3-io`_
- [bed-reader](https://github.com/fastlmm/bed-reader) _简单高效地读取和写入 PLINK BED 格式。_
    - 显示 Rayon/ndarray::parallel（包括捕获错误、控制线程数）、Python 类型到 Rust 泛型、GitHub Actions
- [blake3-py](https://github.com/oconnor663/blake3-py) _[BLAKE3](https://github.com/BLAKE3-team/BLAKE3) 加密哈希函数的 Python 绑定。_
    - 在 GitHub Actions 上并行化[构建](https://github.com/oconnor663/blake3-py/blob/master/.github/workflows/dists.yml)，针对 MacOS、Linux、Windows，包括无线程的 3.13t wheel。
- [cellular_raza](https://cellular-raza.com) _一个基于细胞代理的模拟框架，用于从零开始构建复杂模型。_
- [connector-x](https://github.com/sfu-db/connector-x/tree/main/connectorx-python) _最快的库，用于从 DB 加载数据到 Rust 和 Python 中的 DataFrame。_
- [cryptography](https://github.com/pyca/cryptography/tree/main/src/rust) _Python 加密库，其中一些功能用 Rust 实现。_
- [css-inline](https://github.com/Stranger6667/css-inline/tree/master/bindings/python) _用 Rust 实现的 Python CSS 内联。_
- [datafusion-python](https://github.com/apache/arrow-datafusion-python) _一个绑定到 Apache Arrow 内存查询引擎 DataFusion 的 Python 库。_
- [deltalake-python](https://github.com/delta-io/delta-rs/tree/main/python) _基于 delta-rs 的原生 Delta Lake Python 绑定，带有 Pandas 集成。_
- [fastbloom](https://github.com/yankun1992/fastbloom) _一个快速的 [bloom filter](https://github.com/yankun1992/fastbloom#BloomFilter) | [counting bloom filter](https://github.com/yankun1992/fastbloom#countingbloomfilter)，用 Rust 为 Rust 和 Python 实现！_
- [fastuuid](https://github.com/thedrow/fastuuid/) _Rust 的 UUID 库的 Python 绑定。_
- [feos](https://github.com/feos-org/feos) _Rust 中闪电般快速的热力学建模，带有完全开发的 Python 接口。_
- [finalytics](https://github.com/Nnamdi-sys/finalytics) _Rust | Python 中的投资分析库。_
- [forust](https://github.com/jinlow/forust) _用 Rust 编写的一个轻量级梯度提升决策树库。_
- [geo-index](https://github.com/kylebarron/geo-index) _一个 Rust crate 和 [Python 库](https://github.com/kylebarron/geo-index/tree/main/python)，用于打包的、不可变的、零拷贝空间索引。_
- [granian](https://github.com/emmett-framework/granian) _一个用于 Python 应用程序的 Rust HTTP 服务器。_
- [haem](https://github.com/BooleanCat/haem) _一个用于处理生物信息学问题的 Python 库。_
- [html2text-rs](https://github.com/deedy5/html2text_rs) _将 HTML 转换为标记或纯文本的 Python 库。_
- [html-py-ever](https://github.com/PyO3/setuptools-rust/tree/main/examples/html-py-ever) _通过 [kuchiki](https://github.com/kuchiki-rs/kuchiki) 使用 [html5ever](https://github.com/servo/html5ever) 来加速 HTML 解析和 CSS 选择。_
- [hudi-rs](https://github.com/apache/hudi-rs) _Apache Hudi 的原生 Rust 实现，带有 C++ 和 Python API 绑定。_
- [inline-python](https://github.com/m-ou-se/inline-python) _直接在 Rust 代码中内联 Python 代码。_
- [johnnycanencrypt](https://github.com/kushaldas/johnnycanencrypt) 带有 Yubikey 支持的 OpenPGP 库。
- [jsonschema](https://github.com/Stranger6667/jsonschema/tree/master/crates/jsonschema-py) _一个用于 Python 的高性能 JSON Schema 验证器。_
- [mocpy](https://github.com/cds-astro/mocpy) _天文学 Python 库，提供数据结构，用于描述单位球面上的任意覆盖区域。_
- [obstore](https://github.com/developmentseed/obstore) _最简单的、最高吞吐量的 Python 接口，用于 Amazon S3、Google Cloud Storage、Azure Storage 和其他 S3 兼容 API，由 Rust 驱动。_
- [opendal](https://github.com/apache/opendal/tree/main/bindings/python) _一个数据访问层，允许用户以统一的方式轻松高效地从各种存储服务中检索数据。_
- [orjson](https://github.com/ijl/orjson) _快速的 Python JSON 库。_
- [ormsgpack](https://github.com/aviramha/ormsgpack) _快速的 Python msgpack 库。_
- [polars](https://github.com/pola-rs/polars) _用 Rust | Python | Node.js 实现的快速多线程 DataFrame 库。_
- [pycrdt](https://github.com/jupyter-server/pycrdt) _Rust CRDT 实现 [Yrs](https://github.com/y-crdt/y-crdt) 的 Python 绑定。_
- [pydantic-core](https://github.com/pydantic/pydantic-core) _用 Rust 编写 pydantic 的核心验证逻辑。_
- [primp](https://github.com/deedy5/primp) _最快的 Python HTTP 客户端，能够通过模仿其头部和 TLS/JA3/JA4/HTTP2 指纹来伪装 Web 浏览器。_
- [rateslib](https://github.com/attack68/rateslib) _一个使用 Rust 扩展的 Python 固定收益库。_
- [river](https://github.com/online-ml/river) _Python 中的在线机器学习，计算密集型统计算法用 Rust 实现。_
- [robyn](https://github.com/sparckles/Robyn) 一个具有 Rust 运行时的超级快速异步 Python Web 框架。
- [rust-python-coverage](https://github.com/cjermain/rust-python-coverage) _带有 Rust 和 Python 自动测试覆盖率的 PyO3 项目示例。_
- [rnet](https://github.com/0x676e67/rnet) 带有黑魔法的异步 Python HTTP 客户端
- [sail](https://github.com/lakehq/sail) _统一流、批处理和 AI 工作负载，兼容 Apache Spark。_
- [tiktoken](https://github.com/openai/tiktoken) _一个用于 OpenAI 模型的快速 BPE 分词器。_
- [tokenizers](https://github.com/huggingface/tokenizers/tree/main/bindings/python) _用 Rust 编写的 Hugging Face 分词器（NLP）的 Python 绑定。_
- [tzfpy](http://github.com/ringsaturn/tzfpy) _一个快速将经度/纬度转换为时区名称的包。_
- [utiles](https://github.com/jessekrubin/utiles) _快速的 Python Web 地图图块实用工具_

## 文章和其他媒体

- [(视频) PyO3：从 Python 到 Rust 再返回](https://www.youtube.com/watch?v=UmL_CA-v3O8) - 2024 年 7 月 3 日
- [使用 Rust 将 Python AST 解析速度提高 20 倍](https://www.gauge.sh/blog/parsing-python-asts-20x-faster-with-rust) - 2024 年 6 月 17 日
- [(视频) Python 如何通过 PyO3 利用 Rust](https://www.youtube.com/watch?v=UkZ_m3Wj2hA) - 2024 年 5 月 18 日
- [(视频) 将 Rust 和 Python 结合：两全其美？](https://www.youtube.com/watch?v=lyG6AKzu4ew) - 2024 年 3 月 1 日
- [(视频) 使用 PyO3 扩展 Python 与 Rust](https://www.youtube.com/watch?v=T45ZEmSR1-s) - 2023 年 12 月 16 日
- [PyO3 + rust-numpy 的一周（如何将数据管道速度提高 X 倍）](https://terencezl.github.io/blog/2023/06/06/a-week-of-pyo3-rust-numpy/) - 2023 年 6 月 6 日
- [(播客) 与 David Hewitt 谈论 PyO3](https://rustacean-station.org/episode/david-hewitt/) - 2023 年 5 月 19 日
- [使用不到 100 行 Rust 让 Python 快 100 倍](https://ohadravid.github.io/posts/2023-03-rusty-python/) - 2023 年 3 月 28 日
- [Pydantic V2 如何利用 Rust 的超级能力](https://fosdem.org/2023/schedule/event/rust_how_pydantic_v2_leverages_rusts_superpowers/) - 2023 年 2 月 4 日
- [我们如何使用 PyO3 用 Rust 扩展 River 统计模块](https://boring-guy.sh/posts/river-rust/) - 2022 年 12 月 23 日
- [编写 Rust 中的 Python 扩展的九条规则](https://towardsdatascience.com/nine-rules-for-writing-python-extensions-in-rust-d35ea3a4ec29?sk=f8d808d5f414154fdb811e4137011437) - 2021 年 12 月 31 日
- [使用 PyO3 从 Python 调用 Rust](https://saidvandeklundert.net/learn/2021-11-18-calling-rust-from-python-using-pyo3/) - 2021 年 11 月 18 日
- [davidhewitt 在 2021 年 Rust Manchester 聚会的演讲](https://www.youtube.com/watch?v=-XyWG_klSAw&t=320s) - 2021 年 8 月 19 日
- [逐步将小型 Python 项目移植到 Rust](https://blog.waleedkhan.name/port-python-to-rust/) - 2021 年 4 月 29 日
- [Vortexa - 将 Rust 集成到 Python](https://www.vortexa.com/insight/integrating-rust-into-python) - 2021 年 4 月 12 日
- [编写并发布 Rust 中的 Python 模块](https://blog.yossarian.net/2020/08/02/Writing-and-publishing-a-python-module-in-rust) - 2020 年 8 月 2 日

## 贡献

欢迎每个人为 PyO3 做出贡献！有许多方式来支持该项目，例如：

- 在 GitHub 和 [Discord](https://discord.gg/33kcChzH7f) 上帮助 PyO3 用户解决问题
- 改进文档
- 编写功能和 bug 修复
- 发布关于如何使用 PyO3 的博客和示例

如果您希望为 PyO3 贡献时间并寻找从哪里开始，我们的[贡献指南](contributing.md)和[架构指南](https://github.com/PyO3/pyo3/blob/main/Architecture.md)提供了更多资源。

如果您没有时间亲自贡献，但仍希望支持项目的未来成功，我们的一些维护者有 GitHub 赞助页面：

- [davidhewitt](https://github.com/sponsors/davidhewitt)
- [messense](https://github.com/sponsors/messense)

## 许可

PyO3 根据 [Apache-2.0 许可](LICENSE-APACHE) 或 [MIT 许可](LICENSE-MIT) 许可，由您选择。

Python 根据 [Python 许可](https://docs.python.org/3/license.html) 许可。

除非您明确声明, 否则您有意提交以包含在 PyO3 中的任何贡献，如 Apache 许可中定义，将如上所述双重许可，而无任何附加条款或条件。

<a href="https://www.netlify.com"> <img src="https://www.netlify.com/v3/img/components/netlify-color-accent.svg" alt="Deploys by Netlify" /> </a>
````