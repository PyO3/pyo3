// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::ffi;
use libc::c_int;
use std::ffi::CString;

/// `PyMethodDefType` represents different types of python callable objects.
/// It is used by `#[pymethods]` and `#[pyproto]` annotations.
#[derive(Debug)]
pub enum PyMethodDefType {
    /// Represents class `__new__` method
    New(PyMethodDef),
    /// Represents class `__init__` method
    Init(PyMethodDef),
    /// Represents class `__call__` method
    Call(PyMethodDef),
    /// Represents class method
    Class(PyMethodDef),
    /// Represents static method
    Static(PyMethodDef),
    /// Represents normal method
    Method(PyMethodDef),
    /// Represents getter descriptor, used by `#[getter]`
    Getter(PyGetterDef),
    /// Represents setter descriptor, used by `#[setter]`
    Setter(PySetterDef),
}

#[derive(Copy, Clone, Debug)]
pub enum PyMethodType {
    PyCFunction(ffi::PyCFunction),
    PyCFunctionWithKeywords(ffi::PyCFunctionWithKeywords),
    PyNoArgsFunction(ffi::PyNoArgsFunction),
    PyNewFunc(ffi::newfunc),
    PyInitFunc(ffi::initproc),
}

#[derive(Copy, Clone, Debug)]
pub struct PyMethodDef {
    pub ml_name: &'static str,
    pub ml_meth: PyMethodType,
    pub ml_flags: c_int,
    pub ml_doc: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct PyGetterDef {
    pub name: &'static str,
    pub meth: ffi::getter,
    pub doc: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct PySetterDef {
    pub name: &'static str,
    pub meth: ffi::setter,
    pub doc: &'static str,
}

unsafe impl Sync for PyMethodDef {}

unsafe impl Sync for ffi::PyMethodDef {}

unsafe impl Sync for PyGetterDef {}

unsafe impl Sync for PySetterDef {}

unsafe impl Sync for ffi::PyGetSetDef {}

impl PyMethodDef {
    /// Convert `PyMethodDef` to Python method definition struct `ffi::PyMethodDef`
    pub fn as_method_def(&self) -> ffi::PyMethodDef {
        let meth = match self.ml_meth {
            PyMethodType::PyCFunction(meth) => meth,
            PyMethodType::PyCFunctionWithKeywords(meth) => unsafe { std::mem::transmute(meth) },
            PyMethodType::PyNoArgsFunction(meth) => unsafe { std::mem::transmute(meth) },
            PyMethodType::PyNewFunc(meth) => unsafe { std::mem::transmute(meth) },
            PyMethodType::PyInitFunc(meth) => unsafe { std::mem::transmute(meth) },
        };

        ffi::PyMethodDef {
            ml_name: CString::new(self.ml_name)
                .expect("Method name must not contain NULL byte")
                .into_raw(),
            ml_meth: Some(meth),
            ml_flags: self.ml_flags,
            ml_doc: self.ml_doc.as_ptr() as *const _,
        }
    }
}

impl PyGetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name)
                .expect("Method name must not contain NULL byte")
                .into_raw();
        }
        if dst.doc.is_null() {
            dst.doc = self.doc.as_ptr() as *mut libc::c_char;
        }
        dst.get = Some(self.meth);
    }
}

impl PySetterDef {
    /// Copy descriptor information to `ffi::PyGetSetDef`
    pub fn copy_to(&self, dst: &mut ffi::PyGetSetDef) {
        if dst.name.is_null() {
            dst.name = CString::new(self.name)
                .expect("Method name must not contain NULL byte")
                .into_raw();
        }
        dst.set = Some(self.meth);
    }
}

#[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
/// This trait is implemented for all pyclass to implement the [PyMethodsProtocol]
/// through inventory
pub trait PyMethodsInventoryDispatch {
    /// This allows us to get the inventory type when only the pyclass is in scope
    type InventoryType: PyMethodsInventory;
}

#[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
/// Allows arbitrary pymethod blocks to submit their methods, which are eventually collected by pyclass
pub trait PyMethodsInventory: inventory::Collect {
    /// Create a new instance
    fn new(methods: &'static [PyMethodDefType]) -> Self;

    /// Returns the methods for a single impl block
    fn get_methods(&self) -> &'static [PyMethodDefType];
}

/// The implementation of tis trait defines which methods a python type has.
///
/// For pyclass derived structs this is implemented by collecting all impl blocks through inventory
pub trait PyMethodsProtocol {
    /// Returns all methods that are defined for a class
    fn py_methods() -> Vec<&'static PyMethodDefType>;
}

impl<T> PyMethodsProtocol for T
where
    T: PyMethodsInventoryDispatch,
{
    fn py_methods() -> Vec<&'static PyMethodDefType> {
        inventory::iter::<T::InventoryType>
            .into_iter()
            .flat_map(PyMethodsInventory::get_methods)
            .collect()
    }
}

/// Utils to define and collect dunder methods, powered by inventory
pub mod protocols {
    use crate::ffi;

    #[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
    /// The c wrapper around a dunder method defined in an impl block
    pub enum PyProcotolMethodWrapped {
        Add(ffi::binaryfunc),
    }

    #[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
    /// All defined dunder methods collected into a single struct
    #[derive(Default)]
    pub struct PyProcolTypes {
        pub(crate) add: Option<ffi::binaryfunc>,
    }

    impl PyProcolTypes {
        /// Returns whether any dunder method has been defined
        pub fn any_defined(&self) -> bool {
            self.add.is_some()
        }
    }

    #[doc(hidden)] // Only to be used through the proc macros, use PyMethodsProtocol in custom code
    /// This trait is implemented for all pyclass to implement the [PyProtocolInventory]
    /// through inventory
    pub trait PyProtocolInventoryDispatch {
        /// This allows us to get the inventory type when only the pyclass is in scope
        type ProtocolInventoryType: PyProtocolInventory;
    }

    #[doc(hidden)]
    /// Allows arbitrary pymethod blocks to submit dunder methods, which are eventually collected
    /// into [PyProcolTypes]
    pub trait PyProtocolInventory: inventory::Collect {
        fn new(methods: &'static [PyProcotolMethodWrapped]) -> Self;
        fn get_methods(&self) -> &'static [PyProcotolMethodWrapped];
    }

    /// Defines which protocols this class implements
    pub trait PyProtocol {
        /// Returns all methods that are defined for a class
        fn py_protocols() -> PyProcolTypes;
    }

    impl<T> PyProtocol for T
    where
        T: PyProtocolInventoryDispatch,
    {
        /// Collects all defined dunder methods into a single [PyProcolTypes] instance
        fn py_protocols() -> PyProcolTypes {
            let mut py_protocol_types = PyProcolTypes::default();
            let flattened = inventory::iter::<T::ProtocolInventoryType>
                .into_iter()
                .flat_map(PyProtocolInventory::get_methods);
            for method in flattened {
                match method {
                    PyProcotolMethodWrapped::Add(add) => {
                        if py_protocol_types.add.is_some() {
                            panic!("You can't define `__add__` more than once");
                        }
                        py_protocol_types.add = Some(*add);
                    }
                }
            }

            py_protocol_types
        }
    }
}
