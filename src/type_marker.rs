use crate::conversion::IntoPyPointer;
use crate::ffi;
use crate::pyclass_init::PyObjectInit;
use crate::type_object::{PyLayout, PySizedLayout, PyTypeInfo};
use crate::types::*;
use crate::{AsPyPointer, PyNativeType};

pub trait TypeMarker<'py>: Sized {
    type NativeType: PyNativeType<'py>;
    type Layout: PyLayout<'py, Self>;
    type Initializer: PyObjectInit<'py, Self>;

    /// Type flags (ie PY_TYPE_FLAG_GC, PY_TYPE_FLAG_WEAKREF)
    const FLAGS: usize = 0;

    /// PyTypeObject instance for this type.
    fn type_object() -> &'static ffi::PyTypeObject;

    /// Check if `*mut ffi::PyObject` is instance of this type
    fn is_instance(object: &PyAny) -> bool {
        unsafe {
            ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object() as *const _ as _) != 0
        }
    }

    /// Check if `*mut ffi::PyObject` is exact instance of this type
    fn is_exact_instance(object: &PyAny) -> bool {
        unsafe { (*object.as_ptr()).ob_type == Self::type_object() as *const _ as _ }
    }
}

pub trait TypeWithBase<'py>: TypeMarker<'py> {
    type BaseType: TypeMarker<'py>;
    type BaseNativeType: PyNativeType<'py>;
    type BaseLayout: PySizedLayout<'py, Self::BaseType>;
    type BaseInitializer: PyObjectInit<'py, Self::BaseType>;

    /// PyTypeObject instance for base type.
    fn base_type_object() -> &'static ffi::PyTypeObject {
        Self::BaseType::type_object()
    }
}


#[macro_export]
macro_rules! py_type_marker {
    ($name: ident, $native_ty: ident, $layout: path) => {
        #[derive(Debug)]
        pub struct $name;

        unsafe impl<'py> $crate::type_object::PyLayout<'py, $name> for $layout {}
        unsafe impl<'py> $crate::type_object::PyLayout<'py, $crate::types::$native_ty<'py>> for $layout {}

        impl<'py> TypeMarker<'py> for $name {
            type NativeType = $crate::types::$native_ty<'py>;
            type Layout = $layout;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<'py, Self>;

            /// PyTypeObject instance for this type.
            fn type_object() -> &'static ffi::PyTypeObject {
                <$native_ty<'py> as PyTypeInfo<'py>>::type_object()
            }
        }

        impl<'py> TypeMarker<'py> for $crate::types::$native_ty<'py> {
            type NativeType = $crate::types::$native_ty<'py>;
            type Layout = $layout;
            type Initializer = $crate::pyclass_init::PyNativeTypeInitializer<'py, Self>;

            /// PyTypeObject instance for this type.
            fn type_object() -> &'static ffi::PyTypeObject {
                <$native_ty<'py> as PyTypeInfo<'py>>::type_object()
            }
        }

        impl From<$native_ty<'_>> for $crate::Py<$name> {
            fn from(other: $native_ty<'_>) -> Self {
                unsafe { $crate::Py::from_owned_ptr(other.into_ptr()) }
            }
        }
    };
    ($name: ident, $native_ty: ident) => {
        py_type_marker!($name, $native_ty, ffi::PyObject);
    }
}

py_type_marker!(Any, PyAny);
py_type_marker!(Bool, PyBool);
py_type_marker!(ByteArray, PyByteArray);
py_type_marker!(Bytes, PyBytes);
// py_type_marker!(Iterator, PyIterator);
py_type_marker!(List, PyList);
py_type_marker!(Long, PyLong);
py_type_marker!(Module, PyModule);
py_type_marker!(String, PyString);
py_type_marker!(Tuple, PyTuple);
py_type_marker!(Type, PyType);
py_type_marker!(TzInfo, PyTzInfo);
// py_type_marker!(Sequence, PySequence);

py_type_marker!(Complex, PyComplex, ffi::PyComplexObject);
py_type_marker!(Date, PyDate, ffi::PyDateTime_Date);
py_type_marker!(DateTime, PyDateTime, ffi::PyDateTime_DateTime);
py_type_marker!(Delta, PyDelta, ffi::PyDateTime_Delta);
py_type_marker!(Dict, PyDict, ffi::PyDictObject);
py_type_marker!(Float, PyFloat, ffi::PyFloatObject);
py_type_marker!(FrozenSet, PyFrozenSet, ffi::PySetObject);
py_type_marker!(Set, PySet, ffi::PySetObject);
py_type_marker!(Slice, PySlice, ffi::PySliceObject);
py_type_marker!(Time, PyTime, ffi::PyDateTime_Time);
