use crate::types::*;
use crate::{AsPyPointer, IntoPyPointer, PyNativeType, FromPyPointer};

pub trait Unscoped<'py> {
    type NativeType: PyNativeType<'py> + AsPyPointer + IntoPyPointer + FromPyPointer<'py>;
}

#[macro_export]
macro_rules! py_unscoped {
    ($name: ident, $native_ty: ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl<'py> Unscoped<'py> for $name {
            type NativeType = $native_ty<'py>;
        }

        impl From<$native_ty<'_>> for $crate::Py<$name> {
            fn from(other: $native_ty<'_>) -> Self {
                unsafe { $crate::Py::from_owned_ptr(other.into_ptr()) }
            }
        }
    };
}

py_unscoped!(Dict, PyDict);
py_unscoped!(Tuple, PyTuple);
py_unscoped!(Type, PyType);
