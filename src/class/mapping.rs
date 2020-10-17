// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use crate::callback::IntoPyCallbackOutput;
use crate::{exceptions, ffi, FromPyObject, PyClass, PyObject};

/// Mapping interface
#[allow(unused_variables)]
pub trait PyMappingProtocol<'p>: PyClass {
    fn __len__(&'p self) -> Self::Result
    where
        Self: PyMappingLenProtocol<'p>,
    {
        unimplemented!()
    }

    fn __getitem__(&'p self, key: Self::Key) -> Self::Result
    where
        Self: PyMappingGetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setitem__(&'p mut self, key: Self::Key, value: Self::Value) -> Self::Result
    where
        Self: PyMappingSetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delitem__(&'p mut self, key: Self::Key) -> Self::Result
    where
        Self: PyMappingDelItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __reversed__(&'p self) -> Self::Result
    where
        Self: PyMappingReversedProtocol<'p>,
    {
        unimplemented!()
    }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PyMappingLenProtocol<'p>: PyMappingProtocol<'p> {
    type Result: IntoPyCallbackOutput<usize>;
}

pub trait PyMappingGetItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyMappingSetItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyMappingDelItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyMappingReversedProtocol<'p>: PyMappingProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyMappingSlots {
    fn get_length() -> ffi::PyType_Slot
    where
        Self: for<'p> PyMappingLenProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_mp_length,
            pfunc: py_len_func!(PyMappingLenProtocol, Self::__len__) as _,
        }
    }

    fn get_getitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PyMappingGetItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_mp_subscript,
            pfunc: py_binary_func!(PyMappingGetItemProtocol, Self::__getitem__) as _,
        }
    }

    fn get_setitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PyMappingSetItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_mp_ass_subscript,
            pfunc: py_func_set!(PyMappingSetItemProtocol, Self::__setitem__) as _,
        }
    }

    fn get_delitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PyMappingDelItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_mp_ass_subscript,
            pfunc: py_func_del!(PyMappingDelItemProtocol, Self::__delitem__) as _,
        }
    }

    fn get_setdelitem() -> ffi::PyType_Slot
    where
        Self: for<'p> PyMappingSetItemProtocol<'p> + for<'p> PyMappingDelItemProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_mp_ass_subscript,
            pfunc: py_func_set_del!(
                PyMappingSetItemProtocol,
                PyMappingDelItemProtocol,
                Self,
                __setitem__,
                __delitem__
            ) as _,
        }
    }
}

impl<'p, T> PyMappingSlots for T where T: PyMappingProtocol<'p> {}
