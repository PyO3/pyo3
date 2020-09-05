// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use crate::callback::IntoPyCallbackOutput;
use crate::err::PyErr;
use crate::pyclass::maybe_push_slot;
use crate::{exceptions, ffi, FromPyObject, PyClass, PyObject};
use std::os::raw::c_void;

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

#[derive(Default)]
pub struct PyMappingMethods {
    pub mp_length: Option<ffi::lenfunc>,
    pub mp_subscript: Option<ffi::binaryfunc>,
    pub mp_ass_subscript: Option<ffi::objobjargproc>,
}

#[doc(hidden)]
impl PyMappingMethods {
    pub(crate) fn update_slots(&self, slots: &mut Vec<ffi::PyType_Slot>) {
        maybe_push_slot(
            slots,
            ffi::Py_mp_length,
            self.mp_length.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_mp_subscript,
            self.mp_subscript.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_mp_ass_subscript,
            self.mp_ass_subscript.map(|v| v as *mut c_void),
        );
    }

    pub fn set_length<T>(&mut self)
    where
        T: for<'p> PyMappingLenProtocol<'p>,
    {
        self.mp_length = py_len_func!(PyMappingLenProtocol, T::__len__);
    }
    pub fn set_getitem<T>(&mut self)
    where
        T: for<'p> PyMappingGetItemProtocol<'p>,
    {
        self.mp_subscript = py_binary_func!(PyMappingGetItemProtocol, T::__getitem__);
    }
    pub fn set_setitem<T>(&mut self)
    where
        T: for<'p> PyMappingSetItemProtocol<'p>,
    {
        self.mp_ass_subscript = py_func_set!(PyMappingSetItemProtocol, T, __setitem__);
    }
    pub fn set_delitem<T>(&mut self)
    where
        T: for<'p> PyMappingDelItemProtocol<'p>,
    {
        self.mp_ass_subscript = py_func_del!(PyMappingDelItemProtocol, T, __delitem__);
    }
    pub fn set_setdelitem<T>(&mut self)
    where
        T: for<'p> PyMappingSetItemProtocol<'p> + for<'p> PyMappingDelItemProtocol<'p>,
    {
        self.mp_ass_subscript = py_func_set_del!(
            PyMappingSetItemProtocol,
            PyMappingDelItemProtocol,
            T,
            __setitem__,
            __delitem__
        );
    }
}
