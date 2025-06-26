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

## Setting breakpoints in your Rust code

One of the preferred ways by developers to debug their code is by setting breakpoints. This can be achieved in PyO3 by using a debugger like `rust-gdb` or `rust-lldb` with your Python interpreter.

For more information about how to use both `lldb` and `gdb` you can read the [gdb to lldb command map](https://lldb.llvm.org/use/map.html) from the lldb documentation.

### Common setup

1. Compile your extension with debug symbols:

   ```bash
   # Debug is the default for maturin, but you can explicitly ensure debug symbols with:
   RUSTFLAGS="-g" maturin develop

   # For setuptools-rust users:
   pip install -e .
   ```

   > **Note**: When using debuggers, make sure that `python` resolves to an actual Python binary or symlink and not a shim script. Some tools like pyenv use shim scripts which can interfere with debugging.

### Debugger specific setup

Depending on your OS and your preferences you can use two different debuggers, `rust-gdb` or `rust-lldb`.

{{#tabs }}
{{#tab name="Using rust-gdb" }}

1. Launch rust-gdb with the Python interpreter:

   ```bash
   rust-gdb --args python
   ```

2. Once in gdb, set a breakpoint in your Rust code:

   ```bash
   (gdb) break your_module.rs:42
   ```

3. Run your Python script that imports and uses your Rust extension:

   ```bash
   # Option 1: Run an inline Python command
   (gdb) run -c "import your_module; your_module.your_function()"

   # Option 2: Run a Python script
   (gdb) run your_script.py

   # Option 3: Run pytest tests
   (gdb) run -m pytest tests/test_something.py::TestName
   ```

{{#endtab }}
{{#tab name="Using rust-lldb (for macOS users)" }}

1. Start rust-lldb with Python:

   ```bash
   rust-lldb -- python
   ```

2. Set breakpoints in your Rust code:

   ```bash
   (lldb) breakpoint set --file your_module.rs --line 42
   ```

3. Run your Python script:

   ```bash
   # Option 1: Run an inline Python command
   (lldb) run -c "import your_module; your_module.your_function()"

   # Option 2: Run a Python script
   (lldb) run your_script.py

   # Option 3: Run pytest tests
   (lldb) run -m pytest tests/test_something.py::TestName
   ```

{{#endtab }}
{{#endtabs }}

### Using VS Code

VS Code with the Rust and Python extensions provides an integrated debugging experience:

1. First, install the necessary VS Code extensions:
   * [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
   * [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
   * [Python](https://marketplace.visualstudio.com/items?itemName=ms-python.python)

2. Create a `.vscode/launch.json` file with a configuration that uses the LLDB Debug Launcher:

    ```json
    {
        "version": "0.2.0",
        "configurations": [
            {
                "name": "Debug PyO3",
                "type": "lldb",
                "request": "attach",
                "program": "${workspaceFolder}/.venv/bin/python",
                "pid": "${command:pickProcess}",
                "sourceLanguages": [
                    "rust"
                ]
            },
            {
                "name": "Launch Python with PyO3",
                "type": "lldb",
                "request": "launch",
                "program": "${workspaceFolder}/.venv/bin/python",
                "args": ["${file}"],
                "cwd": "${workspaceFolder}",
                "sourceLanguages": ["rust"]
            },
            {
                "name": "Debug PyO3 with Args",
                "type": "lldb",
                "request": "launch",
                "program": "${workspaceFolder}/.venv/bin/python",
                "args": ["path/to/your/script.py", "arg1", "arg2"],
                "cwd": "${workspaceFolder}",
                "sourceLanguages": ["rust"]
            },
            {
                "name": "Debug PyO3 Tests",
                "type": "lldb",
                "request": "launch",
                "program": "${workspaceFolder}/.venv/bin/python",
                "args": ["-m", "pytest", "tests/your_test.py::test_function", "-v"],
                "cwd": "${workspaceFolder}",
                "sourceLanguages": ["rust"]
            }
        ]
    }
    ```

    This configuration supports multiple debugging scenarios:
    * Attaching to a running Python process
    * Launching the currently open Python file
    * Running a specific script with command-line arguments
    * Running pytest tests

3. Set breakpoints in your Rust code by clicking in the gutter next to line numbers.

4. Start debugging:
   * For attaching to a running Python process: First start the process, then select the "Debug PyO3" configuration and click Start Debugging (F5). You'll be prompted to select the Python process to attach to.
   * For launching a Python script: Open your Python script, select the "Launch Python with PyO3" configuration and click Start Debugging (F5).
   * For running with arguments: Select "Debug PyO3 with Args" (remember to edit the configuration with your actual script path and arguments).
   * For running tests: Select "Debug PyO3 Tests" (edit the test path as needed).

5. When debugging PyO3 code:
   * You can inspect Rust variables and data structures
   * Use the debug console to evaluate expressions
   * Step through Rust code line by line using the step controls
   * Set conditional breakpoints for more complex debugging scenarios

### Advanced Debugging Configurations

For advanced debugging scenarios, you might want to add environment variables or enable specific Rust debug flags:

```json
{
    "name": "Debug PyO3 with Environment",
    "type": "lldb",
    "request": "launch",
    "program": "${workspaceFolder}/.venv/bin/python",
    "args": ["${file}"],
    "env": {
        "RUST_BACKTRACE": "1",
        "PYTHONPATH": "${workspaceFolder}"
    },
    "sourceLanguages": ["rust"]
}
```

### Debugging from Jupyter Notebooks

For Jupyter Notebooks run from VS Code, you can use the following helper functions to automate the launch configuration:

```python
from pathlib import Path
import os
import json
import sys


def update_launch_json(vscode_config_file_path=None):
    """Update VSCode launch.json with the correct Jupyter kernel PID.

    Args:
        vscode_config_file_path (str, optional): Path to the .vscode/launch.json file.
            If not provided, will use the current working directory.
    """
    pid = get_jupyter_kernel_pid()
    if not pid:
        print("Could not determine Jupyter kernel PID.")
        return

    # Determine launch.json path
    if vscode_config_file_path:
        launch_json_path = vscode_config_file_path
    else:
        launch_json_path = os.path.join(Path(os.getcwd()), ".vscode", "launch.json")

    # Get Python interpreter path
    python_path = sys.executable

    # Default debugger config
    debug_config = {
        "version": "0.2.0",
        "configurations": [
            {
                "name": "Debug PyO3 (Jupyter)",
                "type": "lldb",
                "request": "attach",
                "program": python_path,
                "pid": pid,
                "sourceLanguages": ["rust"],
            },
            {
                "name": "Launch Python with PyO3",
                "type": "lldb",
                "request": "launch",
                "program": python_path,
                "args": ["${file}"],
                "cwd": "${workspaceFolder}",
                "sourceLanguages": ["rust"]
            }
        ],
    }

    # Create .vscode directory if it doesn't exist
    try:
        os.makedirs(os.path.dirname(launch_json_path), exist_ok=True)

        # If launch.json already exists, try to update it instead of overwriting
        if os.path.exists(launch_json_path):
            try:
                with open(launch_json_path, "r") as f:
                    existing_config = json.load(f)

                # Check if our configuration already exists
                config_exists = False
                for config in existing_config.get("configurations", []):
                    if config.get("name") == "Debug PyO3 (Jupyter)":
                        config["pid"] = pid
                        config["program"] = python_path
                        config_exists = True

                if not config_exists:
                    existing_config.setdefault("configurations", []).append(debug_config["configurations"][0])

                debug_config = existing_config
            except Exception:
                # If reading fails, we'll just overwrite with our new configuration
                pass

        with open(launch_json_path, "w") as f:
            json.dump(debug_config, f, indent=4)
        print(f"Updated launch.json with PID: {pid} at {launch_json_path}")
    except Exception as e:
        print(f"Error updating launch.json: {e}")


def get_jupyter_kernel_pid():
    """Find the process ID (PID) of the running Jupyter kernel.

    Returns:
        int: The process ID of the Jupyter kernel, or None if not found.
    """
    # Check if we're running in a Jupyter environment
    if 'ipykernel' in sys.modules:
        pid = os.getpid()
        print(f"Jupyter kernel PID: {pid}")
        return pid
    else:
        print("Not running in a Jupyter environment.")
        return None
```

To use these functions:

1. Run the cell containing these functions in your Jupyter notebook
2. Run `update_launch_json()` in a cell
3. In VS Code, select the "Debug PyO3 (Jupyter)" configuration and start debugging


## Thread Safety and Compiler Sanitizers

PyO3 attempts to match the Rust language-level guarantees for thread safety, but
that does not preclude other code outside of the control of PyO3 or buggy code
managed by a PyO3 extension from creating a thread safety issue. Analyzing
whether or not a piece of Rust code that uses the CPython C API is thread safe
can be quite complicated, since many Python operations can lead to arbitrary
Python code execution. Automated ways to discover thread safety issues can often
be more fruitful than code analysis.

[ThreadSanitizer](https://clang.llvm.org/docs/ThreadSanitizer.html) is a thread
safety checking runtime that can be used to detect data races triggered by
thread safety bugs or incorrect use of thread-unsafe data structures. While it
can only detect data races triggered by code at runtime, if it does detect
something the reports often point to exactly where the problem is happening.

To use `ThreadSanitizer` with a library that depends on PyO3, you will need to
install a nightly Rust toolchain, along with the `rust-src` component, since you
will need to compile the Rust standard library:

```bash
rustup install nightly
rustup override set nightly
rustup component add rust-src
```

You will also need a version of CPython compiled using LLVM/Clang with the same
major version of LLVM as is currently used to compile nightly Rust. As of March
2025, Rust nightly uses LLVM 20.

The [cpython_sanity docker images](https://github.com/nascheme/cpython_sanity)
contain a development environment with a pre-compiled version of CPython 3.13 or
3.14 as well as optionally NumPy and SciPy, all compiled using LLVM 20 and
ThreadSanitizer.

After activating a nightly Rust toolchain, you can build your project using
`ThreadSanitizer` with the following command:

```bash
RUSTFLAGS="-Zsanitizer=thread" maturin develop -Zbuild-std --target x86_64-unknown-linux-gnu
```

If you are not running on an x86_64 Linux machine, you should replace
`x86_64-unknown-linux-gnu` with the [target
triple](https://doc.rust-lang.org/rustc/platform-support.html#tier-1-with-host-tools)
that is appropriate for your system. You can also replace `maturin develop` with
`cargo test` to run `cargo` tests. Note that `cargo` runs tests in a thread
pool, so `cargo` tests can be a good way to find thread safety issues.

You can also replace `-Zsanitizer=thread` with `-Zsanitizer=address` or any of
the other sanitizers that are [supported by
Rust](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html). Note
that you'll need to build CPython from source with the appropriate [configure
script
flags](https://docs.python.org/3/using/configure.html#cmdoption-with-address-sanitizer)
to use the same sanitizer environment as you want to use for your Rust
code.
