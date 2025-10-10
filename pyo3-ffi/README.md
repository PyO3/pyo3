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

Requires Rust 1.63 or greater.

`pyo3-ffi` supports the following Python distributions:
  - CPython 3.7 or greater
  - PyPy 7.3 (Python 3.9+)
  - GraalPy 24.0 or greater (Python 3.10+)

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
version = "0.26.0"
features = ["extension-module"]

[build-dependencies]
# This is only necessary if you need to configure your build based on
# the Python version or the compile-time configuration for the interpreter.
pyo3_build_config = "0.26.0"
```

If you need to use conditional compilation based on Python version or how
Python was compiled, you need to add `pyo3-build-config` as a
`build-dependency` in your `Cargo.toml` as in the example above and either
create a new `build.rs` file or modify an existing one so that
`pyo3_build_config::use_pyo3_cfgs()` gets called at build time:

**`build.rs`**

```rust,ignore
fn main() {
    pyo3_build_config::use_pyo3_cfgs()
}
```

**`src/lib.rs`**
```rust,no_run
use std::ffi::{c_char, c_long};
use std::ptr;

use pyo3_ffi::*;

static mut MODULE_DEF: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT,
    m_name: c_str!("string_sum").as_ptr(),
    m_doc: c_str!("A Python module written in Rust.").as_ptr(),
    m_size: 0,
    m_methods: unsafe { METHODS as *const [PyMethodDef] as *mut PyMethodDef },
    m_slots: std::ptr::null_mut(),
    m_traverse: None,
    m_clear: None,
    m_free: None,
};

static mut METHODS: &[PyMethodDef] = &[
    PyMethodDef {
        ml_name: c_str!("sum_as_string").as_ptr(),
        ml_meth: PyMethodDefPointer {
            PyCFunctionFast: sum_as_string,
        },
        ml_flags: METH_FASTCALL,
        ml_doc: c_str!("returns the sum of two integers as a string").as_ptr(),
    },
    // A zeroed PyMethodDef to mark the end of the array.
    PyMethodDef::zeroed(),
];

// The module initialization function, which must be named `PyInit_<your_module>`.
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit_string_sum() -> *mut PyObject {
    let module = PyModule_Create(ptr::addr_of_mut!(MODULE_DEF));
    if module.is_null() {
        return module;
    }
    #[cfg(Py_GIL_DISABLED)]
    {
        if PyUnstable_Module_SetGIL(module, Py_MOD_GIL_NOT_USED) < 0 {
            Py_DECREF(module);
            return std::ptr::null_mut();
        }
    }
    module
}

/// A helper to parse function arguments
/// If we used PyO3's proc macros they'd handle all of this boilerplate for us :)
unsafe fn parse_arg_as_i32(obj: *mut PyObject, n_arg: usize) -> Option<i32> {
    if PyLong_Check(obj) == 0 {
        let msg = format!(
            "sum_as_string expected an int for positional argument {}\0",
            n_arg
        );
        PyErr_SetString(PyExc_TypeError, msg.as_ptr().cast::<c_char>());
        return None;
    }

    // Let's keep the behaviour consistent on platforms where `c_long` is bigger than 32 bits.
    // In particular, it is an i32 on Windows but i64 on most Linux systems
    let mut overflow = 0;
    let i_long: c_long = PyLong_AsLongAndOverflow(obj, &mut overflow);

    #[allow(irrefutable_let_patterns)] // some platforms have c_long equal to i32
    if overflow != 0 {
        raise_overflowerror(obj);
        None
    } else if let Ok(i) = i_long.try_into() {
        Some(i)
    } else {
        raise_overflowerror(obj);
        None
    }
}

unsafe fn raise_overflowerror(obj: *mut PyObject) {
    let obj_repr = PyObject_Str(obj);
    if !obj_repr.is_null() {
        let mut size = 0;
        let p = PyUnicode_AsUTF8AndSize(obj_repr, &mut size);
        if !p.is_null() {
            let s = std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                p.cast::<u8>(),
                size as usize,
            ));
            let msg = format!("cannot fit {} in 32 bits\0", s);

            PyErr_SetString(PyExc_OverflowError, msg.as_ptr().cast::<c_char>());
        }
        Py_DECREF(obj_repr);
    }
}

pub unsafe extern "C" fn sum_as_string(
    _self: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: Py_ssize_t,
) -> *mut PyObject {
    if nargs != 2 {
        PyErr_SetString(
            PyExc_TypeError,
            c_str!("sum_as_string expected 2 positional arguments").as_ptr(),
        );
        return std::ptr::null_mut();
    }

    let (first, second) = (*args, *args.add(1));

    let first = match parse_arg_as_i32(first, 1) {
        Some(x) => x,
        None => return std::ptr::null_mut(),
    };
    let second = match parse_arg_as_i32(second, 2) {
        Some(x) => x,
        None => return std::ptr::null_mut(),
    };

    match first.checked_add(second) {
        Some(sum) => {
            let string = sum.to_string();
            PyUnicode_FromStringAndSize(string.as_ptr().cast::<c_char>(), string.len() as isize)
        }
        None => {
            PyErr_SetString(
                PyExc_OverflowError,
                c_str!("arguments too large to add").as_ptr(),
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
[manual_builds]: https://pyo3.rs/latest/building-and-distribution.html#manual-builds "Manual builds - Building and Distribution - PyO3 user guide"
[setuptools-rust]: https://github.com/PyO3/setuptools-rust "Setuptools plugin for Rust extensions"
[PEP 384]: https://www.python.org/dev/peps/pep-0384 "PEP 384 -- Defining a Stable ABI"
[Features chapter of the guide]: https://pyo3.rs/latest/features.html#features-reference "Features Reference - PyO3 user guide"
