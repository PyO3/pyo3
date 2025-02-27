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

One of the preferred ways by developers to debug their code is by setting breakpoints. This can be achieved in PyO3 by using a debugger like `rust-gdb` or `lldb` with your Python interpreter.
### Using rust-gdb

1. Compile your extension with debug symbols:
   ```bash
   RUSTFLAGS="-g" maturin develop
   ```

2. Launch rust-gdb with the Python interpreter:
   ```bash
   rust-gdb --args python
   ```

3. Once in gdb, set a breakpoint in your Rust code:
   ```
   (gdb) break your_module.rs:42
   ```

4. Run your Python script that imports and uses your Rust extension:
   ```
   (gdb) run -c "import your_module; your_module.your_function()"
   ```

5. When the breakpoint is hit, you can:
   - Print variables: `print variable_name`
   - Step through code: `next` (or `n`) for stepping over, `step` (or `s`) for stepping into
   - Continue execution: `continue` (or `c`)
   - Show the current stack frame: `frame`
   - Show the backtrace: `bt`


### Using rust-lldb (for macOS users)

On macOS, LLDB is the preferred debugger:

1. Compile with debug symbols:
   ```bash
   RUSTFLAGS="-g" maturin develop
   ```

2. Start rust-lldb with Python:
   ```bash
   rust-lldb -- python
   ```

3. Set breakpoints in your Rust code:
   ```
   (lldb) breakpoint set --file your_module.rs --line 42
   ```

4. Run your Python script:
   ```
   (lldb) run -c "import your_module; your_module.your_function()"
   ```

### Using VS Code

VS Code with the Rust and Python extensions provides an integrated debugging experience:

1. Create a `.vscode/launch.json` file with a configuration that uses the LLDB Debug Launcher:

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
            }
        ]
    }
    ```

    This configuration supports both attaching to a running Python process (useful for Jupyter notebooks) and launching a Python script with debugging enabled.

    For Jupyter Notebooks run from VS Code, you'll need to attach to the Jupyter kernel process. Here's an improved version of the helper functions to automate the launch configuration:

    ```python
    from pathlib import Path
    import os
    import subprocess
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

2. Set breakpoints in your Rust code by clicking in the gutter next to line numbers.

3. Start debugging:
   - For attaching to a running Jupyter notebook: First run a cell to ensure the kernel is active, then select the "Debug PyO3 (Jupyter)" configuration and click Start Debugging.
   - For launching a Python script: Open your Python script, select the "Launch Python with PyO3" configuration and click Start Debugging.

4. When debugging PyO3 code:
   - You can inspect Rust variables and data structures
   - Use the debug console to evaluate expressions
   - Step through Rust code line by line using the step controls
   - Set conditional breakpoints for more complex debugging scenarios

For advanced debugging scenarios, you might also want to add environment variables or enable specific Rust debug flags:

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