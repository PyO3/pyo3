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

macro_rules! opaque_native_type_layout {
    ($name:ty $(;$generics:ident)*) => {
        #[cfg(not(PyRustPython))]
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::impl_::pycell::PyVariableClassObjectBase;
            type BaseNativeType = Self;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> =
                $crate::impl_::pycell::PyStaticClassObject<T>;
        }

        #[cfg(PyRustPython)]
        impl<$($generics,)*> $crate::impl_::pyclass::PyClassBaseType for $name {
            type LayoutAsBase = $crate::impl_::pycell::PyVariableClassObjectBase;
            type BaseNativeType = Self;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> =
                $crate::backend::rustpython_storage::PySidecarClassObject<T>;
        }
    };
}
pub(crate) use opaque_native_type_layout;

macro_rules! pyany_native_layout {
    () => {
        #[cfg(not(PyRustPython))]
        impl $crate::impl_::pyclass::PyClassBaseType for $crate::PyAny {
            type LayoutAsBase = $crate::impl_::pycell::PyClassObjectBase<$crate::ffi::PyObject>;
            type BaseNativeType = $crate::PyAny;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> =
                $crate::impl_::pycell::PyStaticClassObject<T>;
        }

        #[cfg(PyRustPython)]
        impl $crate::impl_::pyclass::PyClassBaseType for $crate::PyAny {
            type LayoutAsBase = $crate::impl_::pycell::PyClassObjectBase<$crate::ffi::PyObject>;
            type BaseNativeType = $crate::PyAny;
            type Initializer = $crate::impl_::pyclass_init::PyNativeTypeInitializer<Self>;
            type PyClassMutability = $crate::pycell::impl_::ImmutableClass;
            type Layout<T: $crate::impl_::pyclass::PyClassImpl> =
                $crate::backend::rustpython_storage::PySemanticSidecarClassObject<T>;
        }
    };
}
pub(crate) use pyany_native_layout;

macro_rules! native_exception_subclassable_type {
    ($name:ident, $layout:path) => {
        #[cfg(not(PyRustPython))]
        $crate::pyobject_subclassable_native_type!($name, $layout);

        #[cfg(PyRustPython)]
        $crate::pyobject_subclassable_native_type_opaque!($name);
    };
}
pub(crate) use native_exception_subclassable_type;

macro_rules! pyclass_base_tp_dealloc {
    ($py:expr, $slf:expr, $type_obj:expr) => {{
        #[cfg(PyRustPython)]
        {
            let _ = ($py, $slf, $type_obj);
        }

        #[cfg(not(PyRustPython))]
        unsafe {
            tp_dealloc($slf, $type_obj)
        }
    }};
}
pub(crate) use pyclass_base_tp_dealloc;

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

macro_rules! type_slot_access {
    ($direct:expr, $indirect:block) => {{
        #[cfg(not(any(Py_LIMITED_API, PyRustPython)))]
        {
            $direct
        }

        #[cfg(any(Py_LIMITED_API, PyRustPython))]
        $indirect
    }};
}
pub(crate) use type_slot_access;
