pub mod runtime {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::runtime::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::runtime::*;
}

pub mod err_state {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::err_state::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::err_state::*;
}

pub mod pyclass {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::pyclass::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::pyclass::*;
}

pub mod sync {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::sync::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::sync::*;
}

pub mod string {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::string::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::string::*;
}

pub mod types {
    #[cfg(PyRustPython)]
    pub(crate) use crate::backend::rustpython::types::*;
    #[cfg(not(PyRustPython))]
    pub(crate) use crate::backend::cpython::types::*;
}

macro_rules! dict_subclassable_native_type {
    ($name:ident, $layout:path) => {
        #[cfg(not(any(GraalPy, PyRustPython)))]
        pyobject_native_type_sized!($name, $layout);

        #[cfg(not(any(GraalPy, PyRustPython)))]
        pyobject_subclassable_native_type!($name, $layout);

        #[cfg(PyRustPython)]
        pyobject_subclassable_native_type_opaque!($name);
    };
}
pub(crate) use dict_subclassable_native_type;

macro_rules! set_native_type_decls {
    ($name:ident) => {
        #[cfg(not(any(PyPy, GraalPy, PyRustPython)))]
        pyobject_subclassable_native_type!($name, crate::ffi::PySetObject);

        #[cfg(not(any(PyPy, GraalPy, PyRustPython)))]
        pyobject_native_type!(
            $name,
            ffi::PySetObject,
            pyobject_native_static_type_object!(ffi::PySet_Type),
            "builtins",
            "set",
            #checkfunction=ffi::PySet_Check
        );

        #[cfg(any(PyPy, GraalPy, PyRustPython))]
        pyobject_native_type_core!(
            $name,
            |py| crate::backend::current::types::set_type_object(py),
            "builtins",
            "set",
            #checkfunction=ffi::PySet_Check
        );

        #[cfg(PyRustPython)]
        pyobject_subclassable_native_type_opaque!($name);
    };
}
pub(crate) use set_native_type_decls;

macro_rules! frozenset_native_type_decls {
    ($name:ident) => {
        #[cfg(not(any(PyPy, GraalPy, PyRustPython)))]
        pyobject_subclassable_native_type!($name, crate::ffi::PySetObject);

        #[cfg(not(any(PyPy, GraalPy, PyRustPython)))]
        pyobject_native_type!(
            $name,
            ffi::PySetObject,
            pyobject_native_static_type_object!(ffi::PyFrozenSet_Type),
            "builtins",
            "frozenset",
            #checkfunction=ffi::PyFrozenSet_Check
        );

        #[cfg(any(PyPy, GraalPy, PyRustPython))]
        pyobject_native_type_core!(
            $name,
            |py| crate::backend::current::types::frozenset_type_object(py),
            "builtins",
            "frozenset",
            #checkfunction=ffi::PyFrozenSet_Check
        );
    };
}
pub(crate) use frozenset_native_type_decls;

macro_rules! string_raw_data_api {
    ($($item:item)*) => {
        $(
            #[cfg(not(any(Py_LIMITED_API, GraalPy, PyPy, PyRustPython)))]
            $item
        )*
    };
}
pub(crate) use string_raw_data_api;

macro_rules! string_raw_data_little_endian_test {
    ($item:item) => {
        #[cfg(all(
            not(any(Py_LIMITED_API, PyPy, GraalPy, PyRustPython)),
            target_endian = "little"
        ))]
        $item
    };
}
pub(crate) use string_raw_data_little_endian_test;

macro_rules! tuple_unchecked_item_api {
    ($($item:item)*) => {
        $(
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            $item
        )*
    };
}
pub(crate) use tuple_unchecked_item_api;

macro_rules! tuple_slice_api {
    ($($item:item)*) => {
        $(
            #[cfg(not(any(Py_LIMITED_API, GraalPy, PyRustPython)))]
            $item
        )*
    };
}
pub(crate) use tuple_slice_api;
