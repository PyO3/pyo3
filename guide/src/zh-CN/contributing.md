# 贡献指南

感谢您对贡献 PyO3 的兴趣！欢迎所有人 - 请考虑阅读我们的[行为准则](https://github.com/PyO3/pyo3/blob/main/Code-of-Conduct.md)以维护社区的积极性和包容性。

若您正在寻找贡献方向，请参阅[“开始贡献”](#开始贡献)部分。若已有具体议题需开发，并需要有关开发过程的信息，[“撰写拉取请求”](#撰写拉取请求)部分将提供流程指导。

如果您想熟悉代码库，请参阅[Architecture.md](https://github.com/PyO3/pyo3/blob/main/Architecture.md)。

## 开始贡献

请加入您对 PyO3 感兴趣的的任何部分。我们使用 GitHub 问题来记录所有错误和想法。如果您想处理某个具体问题，请随时请求将其分配给您。

您可以[在这里](https://pyo3.netlify.app/internal/doc/pyo3/index.html)浏览 PyO3 非公共部分的 API。

以下部分将包括具体的贡献方向建议。

## 设置开发环境

为了使用和开发 PyO3，您需要在系统上安装 Python 和 Rust。
* 我们推荐使用 [rustup](https://rustup.rs/) 灵活管理项目所需的 Rust 工具链。
* 强烈建议通过 [Pyenv](https://github.com/pyenv/pyenv) 选择特定 Python 版本。。
* 可搭配 [virtualenv](https://virtualenv.pypa.io/en/latest/) （独立或与 Pyenv 协同）调用指定 Python 版本。
* 自动化 CI 任务依赖 [`nox`][nox] 工具实现。

### 帮助用户识别错误

[PyO3 Discord 服务器](https://discord.gg/33kcChzH7f) 非常活跃，有许多 PyO3 新用户，而且往往完全是 Rust 新手。帮助他们调试是获得 PyO3 代码库经验的好方法。

帮助他人往往会揭示当前错误、文档弱点和缺失的 API。建议立即为这些创建 GitHub Issues，以便解决方案可以被设计和实现！

### 实现准备开发的 issue

解决方案明确但尚未实现的 issue 使用 [needs-implementer](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-implementer) 标签。

如果您对解决方案感到困惑，不用担心！PyO3 的核心贡献者将很乐意指导您解答任何问题，以帮助您编写解决方案。

### 帮助编写优秀的文档

PyO3 提供采用 mdBook 构建的用户指南以及常规 Rust API 文档。两者均致力于实现详实准确、通俗易懂且及时更新。我们始终欢迎提交 PR 以修复拼写错误、优化措辞、补充示例等。

当前需要重点支持的文档工作领域：

- 请求文档改进的 issue 使用 [documentation](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Adocumentation) 标签跟踪。
- 并非所有 API 在创建时都有文档或示例。我们的目标是为所有 PyO3 API 提供文档 [](https://github.com/PyO3/pyo3/issues/306)。如果您看到 API 缺少文档，可以补充编写并打开 PR！

如需构建文档（包括所有功能），请安装 [`nox`][nox] 然后运行

```shell
nox -s docs -- open
```

#### 文档测试

我们在文档中使用了许多代码块。在进行更改时运行 `cargo test --doc` 来检查文档测试是否仍然有效，或者 `cargo test` 来运行所有 Rust 测试，包括文档测试。请参阅 https://doc.rust-lang.org/rustdoc/documentation-tests.html 获取文档测试指南。

#### 构建指南

您可以使用 `mdbook` 在本地构建用户指南来预览它。

首先，安装 [`mdbook`][mdbook]、[`mdbook-tabs`][mdbook-tabs] 插件和 [`nox`][nox]。然后，运行

```shell
nox -s build-guide -- --open
```

如需检查指南中的所有链接是否有效，请安装 [`lychee`][lychee] 并使用 `check-guide` 会话：

```shell
nox -s check-guide
```

### 帮助设计下一个 PyO3

尚未有明确解决方案的 issue 使用 [needs-design](https://github.com/PyO3/pyo3/issues?q=is%3Aissue+is%3Aopen+label%3Aneeds-design) 标签。

若您对以上任何议题感兴趣，​欢迎加入相关议题的讨论​！所有意见都备受重视。若您愿意进一步参与（例如通过草稿PR尝试API设计），那会更棒！

### 审查拉取请求

欢迎每个人在开放的 PR 上提交评论。请帮助确保新的 PyO3 API 安全、高性能、整洁且易用！

## 撰写拉取请求

在编写 PR 时要注意的几件事。

### 测试和持续集成

PyO3 仓库使用 GitHub Actions。如果 CI 不成功，PR 将被阻止合并。对所有 Rust 和 Python 代码检查格式化、linting 和测试。此外，Rust 代码中的所有警告都被禁止（使用 `RUSTFLAGS="-D warnings"`）。

测试使用所有支持的 Python 版本与最新的稳定 Rust 编译器运行，以及 Python 3.9 与最低支持的 Rust 版本。

如果您添加新功能，您应该将其添加到我们的 *Cargo.toml* 中的 `full` 功能，以便在 CI 中测试。

您可以使用 `nox` 自己运行这些检查。使用 `nox -l` 列出您可以运行的完整子命令集。

#### Linting Python 代码
`nox -s ruff`

#### Linting Rust 代码
`nox -s rustfmt`

#### Semver 检查
`cargo semver-checks check-release`

#### Clippy
`nox -s clippy-all`

#### 测试
`nox -s test` 或仅 Rust 测试的 `cargo test`，仅 Python 测试的 `nox -f pytests/noxfile.py -s test`

#### 检查所有条件编译
`nox -s check-feature-powerset`

#### UI 测试

PyO3 使用 [`trybuild`](https://github.com/dtolnay/trybuild) 开发 UI 测试，以捕获 Rust 编译器的一些宏功能的错误消息。

因为这些 UI 测试有几个功能组合，当更新它们所有（例如对于新的 Rust 编译器版本）时，使用 `update-ui-tests` nox 会话可能会有帮助：

```bash
nox -s update-ui-tests
```

### 文档更改

我们使用 [towncrier](https://towncrier.readthedocs.io/en/stable/index.html) 为每个发布生成 CHANGELOG。

要在发布笔记中包含您的更改，您应该在 `newsfragments` 目录中创建一个（或多个）新闻项。有效的新闻项应保存为 `<PR>.<CATEGORY>.md`，其中 `<PR>` 是拉取请求编号，`<CATEGORY>` 是以下之一：
- `packaging` - 用于依赖更改和 Python / Rust 版本兼容性更改
- `added` - 用于新功能
- `changed` - 用于已存在但已被更改或弃用的功能
- `removed` - 用于已移除的功能
- `fixed` - 用于被分类为 bug 修复的“changed”功能

仅文档 PR 不需要新闻项；以 `docs:` 开头您的 PR 标题以跳过检查。

### 风格指南

#### 泛型代码

PyO3 有许多泛型 API 来提高可用性。这些可能以泛型代码膨胀为代价。在合理的情况下，尝试实现泛型函数的具体子部分。有两种形式：

- 如果具体子部分不受益于其他函数的重用，将其命名为 `inner` 并保持为函数的局部。
- 如果具体子部分被其他函数重用，最好将其命名为 `_foo` 并放置在源代码中 `foo` 的正下方（其中 `foo` 是原始泛型函数）。

#### FFI 调用

PyO3 使用原始指针对 Python 的 C API 进行许多 FFI 调用。在可能的情况下，尝试避免在表达式中使用指向临时变量的指针：

```rust
// dangerous
pyo3::ffi::Something(name.to_object(py).as_ptr());

// because the following refactoring is a use-after-free error:
let name = name.to_object(py).as_ptr();
pyo3::ffi::Something(name)
```

相反，优先绑定安全的拥有的 `PyObject` 包装器，然后传递给 FFI 函数：

```rust
let name: PyObject = name.to_object(py);
pyo3::ffi::Something(name.as_ptr())
// name will automatically be freed when it falls out of scope
```

## Python 和 Rust 版本支持政策

PyO3 致力于充分保持兼容性，确保使用 PyO3 构建的 Python 扩展能够在大多数常见软件包管理器上顺利部署。

为尽可能减轻软件包维护者的工作负担，PyO3 承诺在可行范围内，​仅当同步调整最低支持的 Rust 和 Python 版本时，才会更新兼容性要求。此类调整仅发生在 0.x 版本发布中（更新频率约为每年一次），且仅在旧版 Python 结束生命周期后实施。（具体时间表请参考 Python 官方生命周期日历：https://endoflife.date/python）

以下列示了 PR 的语言兼容性指南。

### Python

PyO3 支持所有官方支持的 Python 版本，以及最新的 PyPy3 发布。所有这些版本都在 CI 中测试。

#### 添加对新 CPython 版本的支持

如果您计划添加对 CPython 预发布版本的支持，这里是一个（非详尽的）检查列表：

 - [ ] 等到最后一个 alpha 发布（通常是 alpha7），因为 ABI 直到第一个 beta 发布才保证
 - [ ] 将 prelease_ver-dev（例如 `3.14-dev`）添加到 `.github/workflows/ci.yml`，并在 `noxfile.py`、`pyo3-ffi/Cargo.toml` 中的 `[package.metadata.cpython]` 下的 `max-version` 和 `pyo3-ffi/build.rs` 中的 `max` 中提升版本
- [ ] 为版本添加新的 abi3-prerelease 功能（例如 `abi3-py314`）
   - 在 `pyo3-build-config/Cargo.toml` 中，将 abi3-most_current_stable 设置为 ["abi3-prerelease"]，abi3-prerelease 设置为 ["abi3"]
   - 在 `pyo3-ffi/Cargo.toml` 中，将 abi3-most_current_stable 设置为 ["abi3-prerelease", "pyo3-build-config/abi3-most_current_stable"]，abi3-prerelease 设置为 ["abi3", "pyo3-build-config/abi3-prerelease"]
   - 在 `Cargo.toml` 中，将 abi3-most_current_stable 设置为 ["abi3-prerelease", "pyo3-ffi/abi3-most_current_stable"]，abi3-prerelease 设置为 ["abi3", "pyo3-ffi/abi3-prerelease"]
 - [ ] 使用 `#[cfg(Py_prerelease)]`（例如 `#[cfg(Py_3_14)]`）和 `#[cfg(not(Py_prerelease))]` 来指示 CPython 稳定分支和预发布之间的更改
 - [ ] 不要为 CPython 头文件中以 `_` 前缀的任何函数、结构体或全局变量添加 Rust 绑定
 - [ ] Ping @ngoldbaum 和 @davidhewitt 以获取帮助

### Rust

PyO3 致力于充分利用 Rust 语言的最新特性，确保底层实现始终保持最高效率。

所支持的最低 Rust 版本将在升级 Python 和 Rust 依赖版本时确定。具体策略是：将最低 Rust 版本设置为不超过当前 Debian、RHEL 和 Alpine Linux 发行版预装的最低 Rust 版本。

持续集成（CI）系统会同步测试最新的稳定 Rust 版本和已确定的最低支持版本。得益于 Rust 的稳定性保障机制，此策略可确保所有中间版本均获得兼容性支持。

## 基准测试

PyO3 提供两套基准测试方案，用于评估关键性能指标。当前测试覆盖范围有限，​欢迎通过 PR 补充新测试用例以扩展该体系！

首先，有位于 `pyo3-benches` 子目录中的基于 Rust 的基准测试。您可以使用以下命令运行这些基准测试：

    nox -s bench

其次，在 `pytests` 子目录中有一个基于 Python 的基准测试。您可以[在这里](https://github.com/PyO3/pyo3/tree/main/pytests)阅读更多关于它的信息。

## 代码覆盖率

您可以查看 PyO3 测试覆盖和未覆盖的代码。我们旨在达到 100% 覆盖率 - 如果您注意到缺乏覆盖，请检查覆盖率并添加测试！

- 首先，确保安装了 llvm-cov cargo 插件。在与 `nox` 一起使用之前，您可能需要通过 cargo 运行一次插件。
```shell
cargo install cargo-llvm-cov
cargo llvm-cov
```
- 然后，使用以下命令生成 `lcov.info` 文件
```shell
nox -s coverage -- lcov
```
您可以安装 IDE 插件来查看覆盖率。例如，如果您使用 VSCode：
- 添加 [coverage-gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) 插件。
- 将这些设置添加到 VSCode 的 `settings.json`：
```json
{
    "coverage-gutters.coverageFileNames": [
        "lcov.info",
        "cov.xml",
        "coverage.xml",
    ],
    "coverage-gutters.showLineCoverage": true
}
```
- 您现在应该能够看到测试代码的绿色突出显示，以及未测试代码的红色突出显示。

## 赞助此项目

目前没有官方组织代表 PyO3 接受赞助。如果您寻求为 PyO3 生态系统提供大量资金，请在 [GitHub](https://github.com/PyO3/pyo3/issues/new) 或 [Discord](https://discord.gg/33kcChzH7f) 联系我们，我们可以讨论。

与此同时，我们的一些维护者有个人 GitHub 赞助页面，社区将非常感激您的支持：

- [davidhewitt](https://github.com/sponsors/davidhewitt)
- [messense](https://github.com/sponsors/messense)

[mdbook]: https://rust-lang.github.io/mdBook/cli/index.html
[mdbook-tabs]: https://mdbook-plugins.rustforweb.org/tabs.html
[lychee]: https://github.com/lycheeverse/lychee
[nox]: https://github.com/theacodes/nox