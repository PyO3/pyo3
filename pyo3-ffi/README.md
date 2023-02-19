# pyo3-ffi

This crate provides [Rust](https://www.rust-lang.org/) FFI declarations for Python 3.
It supports both the stable and the unstable component of the ABI through the use of cfg flags.
Python Versions 3.7+ are supported.
It is meant for advanced users only - regular PyO3 users shouldn't
need to interact with this crate at all.

The contents of this crate are not documented here, as it would entail
basically copying the documentation from CPython. Consult the [Python/C API Reference
Manual][capi] for up-to-date documentation.

# Minimum supported Rust and Python versions

PyO3 supports the following software versions:
  - Python 3.7 and up (CPython and PyPy)
  - Rust 1.48 and up

# Example: Building Python Native modules

PyO3 can be used to generate a native Python module. The easiest way to try this out for the
first time is to use [`maturin`]. `maturin` is a tool for building and publishing Rust-based
Python packages with minimal configuration. The following steps set up some files for an example
Python module, install `maturin`, and then show how to build and import the Python module.

First, create a new folder (let's call it `string_sum`) containing the following two files:

**`Cargo.toml`**

```toml
[lib]
name = "string_sum"
# "cdylib" is necessary to produce a shared library for Python to import from.
#
# Downstream Rust code (including code in `bin/`, `examples/`, and `tests/`) will not be able
# to `use string_sum;` unless the "rlib" or "lib" crate type is also included, e.g.:
# crate-type = ["cdylib", "rlib"]
crate-type = ["cdylib"]

[dependencies.pyo3-ffi]
version = "*"
features = ["native-module"]
```

**`src/lib.rs`**
```rust
use std::os::raw::c_char;
use std::ptr;

use pyo3_ffi::*;

static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT,
    m_name: "string_sum\0".as_ptr().cast::<c_char>(),
    m_doc: "A Python module written in Rust.\0"
        .as_ptr()
        .cast::<c_char>(),
    m_size: 0,
    m_methods: unsafe { METHODS.as_mut_ptr().cast() },
    m_slots: std::ptr::null_mut(),
    m_traverse: None,
    m_clear: None,
    m_free: None,
};

static mut METHODS: [PyMethodDef; 2] = [
    PyMethodDef {
        ml_name: "sum_as_string\0".as_ptr().cast::<c_char>(),
        ml_meth: PyMethodDefPointer {
            _PyCFunctionFast: sum_as_string,
        },
        ml_flags: METH_FASTCALL,
        ml_doc: "returns the sum of two integers as a string\0"
            .as_ptr()
            .cast::<c_char>(),
    },
    // A zeroed PyMethodDef to mark the end of the array.
    PyMethodDef::zeroed()
];

// The module initialization function, which must be named `PyInit_<your_module>`.
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
    PyModule_Create(ptr::addr_of_mut!(MODULE_DEF))
}

pub unsafe extern "C" fn sum_as_string(
    _self: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: Py_ssize_t,
) -> *mut PyObject {
    if nargs != 2 {
        PyErr_SetString(
            PyExc_TypeError,
            "sum_as_string() expected 2 positional arguments\0"
                .as_ptr()
                .cast::<c_char>(),
        );
        return std::ptr::null_mut();
    }

    let arg1 = *args;
    if PyLong_Check(arg1) == 0 {
        PyErr_SetString(
            PyExc_TypeError,
            "sum_as_string() expected an int for positional argument 1\0"
                .as_ptr()
                .cast::<c_char>(),
        );
        return std::ptr::null_mut();
    }

    let arg1 = PyLong_AsLong(arg1);
    if !PyErr_Occurred().is_null() {
        return ptr::null_mut();
    }

    let arg2 = *args.add(1);
    if PyLong_Check(arg2) == 0 {
        PyErr_SetString(
            PyExc_TypeError,
            "sum_as_string() expected an int for positional argument 2\0"
                .as_ptr()
                .cast::<c_char>(),
        );
        return std::ptr::null_mut();
    }

    let arg2 = PyLong_AsLong(arg2);
    if !PyErr_Occurred().is_null() {
        return ptr::null_mut();
    }

    match arg1.checked_add(arg2) {
        Some(sum) => {
            let string = sum.to_string();
            PyUnicode_FromStringAndSize(string.as_ptr().cast::<c_char>(), string.len() as isize)
        }
        None => {
            PyErr_SetString(
                PyExc_OverflowError,
                "arguments too large to add\0".as_ptr().cast::<c_char>(),
            );
            std::ptr::null_mut()
        }
    }
}
```

With those two files in place, now `maturin` needs to be installed. This can be done using
Python's package manager `pip`. First, load up a new Python `virtualenv`, and install `maturin`
into it:
```bash
$ cd string_sum
$ python -m venv .env
$ source .env/bin/activate
$ pip install maturin
```

Now build and execute the module:
```bash
$ maturin develop
# lots of progress output as maturin runs the compilation...
$ python
>>> import string_sum
>>> string_sum.sum_as_string(5, 20)
'25'
```

As well as with `maturin`, it is possible to build using [setuptools-rust] or
[manually][manual_builds]. Both offer more flexibility than `maturin` but require further
configuration.


While most projects use the safe wrapper provided by PyO3,
you can take a look at the [`orjson`] library as an example on how to use `pyo3-ffi` directly.
For those well versed in C and Rust the [tutorials] from the CPython documentation
can be easily converted to rust as well.

[tutorials]: https://docs.python.org/3/extending/
[`orjson`]: https://github.com/ijl/orjson
[capi]: https://docs.python.org/3/c-api/index.html
[`maturin`]: https://github.com/PyO3/maturin "Build and publish crates with pyo3, rust-cpython and cffi bindings as well as rust binaries as python packages"
[`pyo3-build-config`]: https://docs.rs/pyo3-build-config
[feature flags]: https://doc.rust-lang.org/cargo/reference/features.html "Features - The Cargo Book"
[manual_builds]: https://pyo3.rs/latest/building_and_distribution.html#manual-builds "Manual builds - Building and Distribution - PyO3 user guide"
[setuptools-rust]: https://github.com/PyO3/setuptools-rust "Setuptools plugin for Rust extensions"
[PEP 384]: https://www.python.org/dev/peps/pep-0384 "PEP 384 -- Defining a Stable ABI"
[Features chapter of the guide]: https://pyo3.rs/latest/features.html#features-reference "Features Reference - PyO3 user guide"
