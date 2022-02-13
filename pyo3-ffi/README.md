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
features = ["extension-module"]
```

**`src/lib.rs`**
```rust
use std::os::raw::c_char;
use std::ptr;

use pyo3_ffi::*;

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
    let init = PyModuleDef {
        m_base: PyModuleDef_HEAD_INIT,
        m_name: "string_sum\0".as_ptr() as *const c_char,
        m_doc: std::ptr::null(),
        m_size: 0,
        m_methods: std::ptr::null_mut(),
        m_slots: std::ptr::null_mut(),
        m_traverse: None,
        m_clear: None,
        m_free: None,
    };

    let mptr = PyModule_Create(Box::into_raw(Box::new(init)));
    let version = env!("CARGO_PKG_VERSION");
    PyModule_AddObject(
        mptr,
        "__version__\0".as_ptr() as *const c_char,
        PyUnicode_FromStringAndSize(version.as_ptr() as *const c_char, version.len() as isize),
    );

    let wrapped_sum_as_string = PyMethodDef {
        ml_name: "sum_as_string\0".as_ptr() as *const c_char,
        ml_meth: MlMeth {
            _PyCFunctionFast: Some(sum_as_string)
        },
        ml_flags: METH_FASTCALL,
        ml_doc: "returns the sum of two integers as a string\0".as_ptr() as *const c_char,
    };

    // PyModule_AddObject can technically fail.
    // For more involved applications error checking may be necessary
    PyModule_AddObject(
        mptr,
        "sum_as_string\0".as_ptr() as *const c_char,
        PyCFunction_NewEx(
            Box::into_raw(Box::new(wrapped_sum_as_string)),
            std::ptr::null_mut(),
            PyUnicode_InternFromString("string_sum\0".as_ptr() as *const c_char),
        ),
    );

    let all = ["__all__\0", "__version__\0", "sum_as_string\0"];

    let pyall = PyTuple_New(all.len() as isize);
    for (i, obj) in all.iter().enumerate() {
        PyTuple_SET_ITEM(
            pyall,
            i as isize,
            PyUnicode_InternFromString(obj.as_ptr() as *const c_char),
        )
    }

    PyModule_AddObject(mptr, "__all__\0".as_ptr() as *const c_char, pyall);

    mptr
}

pub unsafe extern "C" fn sum_as_string(
    _self: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: Py_ssize_t,
) -> *mut PyObject {
    if nargs != 2 {
        return raise_type_error("sum_as_string() expected 2 positional arguments");
    }

    let arg1 = *args;
    if PyLong_Check(arg1) == 0 {
        return raise_type_error("sum_as_string() expected an int for positional argument 1");
    }

    let arg1 = PyLong_AsLong(arg1);
    if !PyErr_Occurred().is_null() {
        return ptr::null_mut()
    }

    let arg2 = *args.add(1);
    if PyLong_Check(arg2) == 0 {
        return raise_type_error("sum_as_string() expected an int for positional argument 2");
    }

    let arg2 = PyLong_AsLong(arg2);
    if !PyErr_Occurred().is_null() {
        return ptr::null_mut()
    }



    let res = (arg1 + arg2).to_string();
    PyUnicode_FromStringAndSize(res.as_ptr() as *const c_char, res.len() as isize)
}

#[cold]
#[inline(never)]
fn raise_type_error(msg: &str) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        PyErr_SetObject(PyExc_TypeError, err_msg);
        Py_DECREF(err_msg);
    };
    std::ptr::null_mut()
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
