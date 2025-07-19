# Logging

It is desirable if both the Python and Rust parts of the application end up
logging using the same configuration into the same place.

This section of the guide briefly discusses how to connect the two languages'
logging ecosystems together. The recommended way for Python extension modules is
to configure Rust's logger to send log messages to Python using the `pyo3-log`
crate. For users who want to do the opposite and send Python log messages to
Rust, see the note at the end of this guide.

## Using `pyo3-log` to send Rust log messages to Python

The [pyo3-log] crate allows sending the messages from the Rust side to Python's
[logging] system. This is mostly suitable for writing native extensions for
Python programs.

Use [`pyo3_log::init`][init] to install the logger in its default configuration.
It's also possible to tweak its configuration (mostly to tune its performance).

```rust,no_run
#[pyo3::pymodule]
mod my_module {
    use log::info;
    use pyo3::prelude::*;

    #[pyfunction]
    fn log_something() {
        // This will use the logger installed in `my_module` to send the `info`
        // message to the Python logging facilities.
        info!("Something!");
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // A good place to install the Rust -> Python logger.
        pyo3_log::init();
    }
}
```

Then it is up to the Python side to actually output the messages somewhere.

```python
import logging
import my_module

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.INFO)
my_module.log_something()
```

It is important to initialize the Python loggers first, before calling any Rust
functions that may log. This limitation can be worked around if it is not
possible to satisfy, read the documentation about [caching].

## The Python to Rust direction

To have python logs be handled by Rust, one need only register a rust function to handle logs emitted from the core python logging module.

This has been implemented within the [pyo3-pylogger] crate.

```rust,no_run
use log::{info, warn};
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    // register the host handler with python logger, providing a logger target
    // set the name here to something appropriate for your application
    pyo3_pylogger::register("example_application_py_logger");

    // initialize up a logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // Log some messages from Rust.
    info!("Just some normal information!");
    warn!("Something spooky happened!");

    // Log some messages from Python
    Python::attach(|py| {
        py.run(
            "
import logging
logging.error('Something bad happened')
",
            None,
            None,
        )
    })
}
```

[logging]: https://docs.python.org/3/library/logging.html
[pyo3-log]: https://crates.io/crates/pyo3-log
[init]: https://docs.rs/pyo3-log/*/pyo3_log/fn.init.html
[caching]: https://docs.rs/pyo3-log/*/pyo3_log/#performance-filtering-and-caching
[pyo3-pylogger]: https://crates.io/crates/pyo3-pylogger
