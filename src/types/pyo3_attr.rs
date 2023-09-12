use std::ffi::CStr;

use crate::class::PyMethodDefType;
use crate::ffi::{reprfunc, PyTypeObject, PyType_Slot, Py_tp_repr};
use crate::impl_::{
    extract_argument::{extract_pyclass_ref, PyFunctionArgument},
    pycell::PyClassMutability,
    pyclass::{
        LazyTypeObject, PyClassBaseType, PyClassDummySlot, PyClassImpl, PyClassImplCollector,
        PyClassItems, PyClassItemsIter, PyMethods, SendablePyClass,
    },
    trampoline::reprfunc,
};
use crate::intern;
use crate::methods::PyClassAttributeFactory;
use crate::pyclass::boolean_struct;
use crate::{IntoPy, Py, PyAny, PyCell, PyClass, PyObject, PyRef, PyResult, PyTypeInfo, Python};

pub(crate) struct PyO3Attr {}

unsafe impl PyTypeInfo for PyO3Attr {
    type AsRefTarget = PyCell<Self>;
    const NAME: &'static str = "_pyo3_internal.PyO3Attr";
    const MODULE: Option<&'static str> = None;

    #[inline]
    fn type_object_raw(py: Python<'_>) -> *mut PyTypeObject {
        <Self as PyClassImpl>::lazy_type_object()
            .get_or_init(py)
            .as_type_ptr()
    }
}

impl PyClass for PyO3Attr {
    type Frozen = boolean_struct::True;
}

impl<'a, 'py> PyFunctionArgument<'a, 'py> for &'a PyO3Attr {
    type Holder = Option<PyRef<'py, PyO3Attr>>;

    #[inline]
    fn extract(obj: &'py PyAny, holder: &'a mut Self::Holder) -> PyResult<Self> {
        extract_pyclass_ref(obj, holder)
    }
}

impl IntoPy<PyObject> for PyO3Attr {
    fn into_py(self, py: Python<'_>) -> PyObject {
        IntoPy::into_py(Py::new(py, self).unwrap(), py)
    }
}

impl PyClassImpl for PyO3Attr {
    const IS_BASETYPE: bool = true;
    type BaseType = PyAny;
    type ThreadChecker = SendablePyClass<PyO3Attr>;
    type PyClassMutability =
        <<Self::BaseType as PyClassBaseType>::PyClassMutability as PyClassMutability>::ImmutableChild;
    type Dict = PyClassDummySlot;
    type WeakRef = PyClassDummySlot;
    type BaseNativeType = Self::BaseType;

    fn items_iter() -> PyClassItemsIter {
        let collector = PyClassImplCollector::<Self>::new();
        static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
            methods: &[PyMethodDefType::ClassAttribute(
                crate::PyClassAttributeDef::new(
                    "version\0",
                    PyClassAttributeFactory(PyO3Attr::version),
                ),
            )],
            slots: &[{
                unsafe extern "C" fn trampoline(
                    _slf: *mut crate::ffi::PyObject,
                ) -> *mut crate::ffi::PyObject {
                    reprfunc(_slf, PyO3Attr::__repr__)
                }
                PyType_Slot {
                    slot: Py_tp_repr,
                    pfunc: trampoline as reprfunc as _,
                }
            }],
        };
        PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
    }

    fn doc(_py: Python<'_>) -> PyResult<&'static CStr> {
        Ok(Self::DOC)
    }

    fn lazy_type_object() -> &'static LazyTypeObject<Self> {
        static TYPE_OBJECT: LazyTypeObject<PyO3Attr> = LazyTypeObject::new();
        &TYPE_OBJECT
    }
}

impl PyO3Attr {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const DOC: &'static CStr =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"The PyO3 information for a module.\0") };

    fn version(py: Python<'_>) -> PyResult<PyObject> {
        Ok(intern!(py, PyO3Attr::VERSION).into_py(py))
    }

    unsafe fn __repr__(
        py: Python<'_>,
        _raw_slf: *mut crate::ffi::PyObject,
    ) -> PyResult<*mut crate::ffi::PyObject> {
        crate::callback::convert(py, intern!(py, <PyO3Attr as PyTypeInfo>::NAME))
    }
}

#[cfg(test)]
mod test {
    use super::PyO3Attr;
    use crate::{IntoPy, Python};

    #[test]
    fn test_version_str() {
        Python::with_gil(|py| {
            let attr = IntoPy::into_py(PyO3Attr {}, py);
            assert_eq!(
                attr.getattr(py, "version")
                    .unwrap()
                    .extract::<&str>(py)
                    .unwrap(),
                env!("CARGO_PKG_VERSION")
            )
        })
    }
}
