// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! [Python information](
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors)

use crate::callback::IntoPyCallbackOutput;
use crate::pyclass::maybe_push_slot;
use crate::types::PyAny;
use crate::{ffi, FromPyObject, PyClass, PyObject};
use std::os::raw::{c_int, c_void};

/// Descriptor interface
#[allow(unused_variables)]
pub trait PyDescrProtocol<'p>: PyClass {
    fn __get__(
        slf: Self::Receiver,
        instance: Self::Inst,
        owner: Option<Self::Owner>,
    ) -> Self::Result
    where
        Self: PyDescrGetProtocol<'p>,
    {
        unimplemented!()
    }

    fn __set__(slf: Self::Receiver, instance: Self::Inst, value: Self::Value) -> Self::Result
    where
        Self: PyDescrSetProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delete__(&'p self, instance: &'p PyAny) -> Self::Result
    where
        Self: PyDescrDeleteProtocol<'p>,
    {
        unimplemented!()
    }

    fn __set_name__(&'p self, instance: &'p PyAny) -> Self::Result
    where
        Self: PyDescrSetNameProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyDescrGetProtocol<'p>: PyDescrProtocol<'p> {
    type Receiver: crate::derive_utils::TryFromPyCell<'p, Self>;
    type Inst: FromPyObject<'p>;
    type Owner: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyDescrSetProtocol<'p>: PyDescrProtocol<'p> {
    type Receiver: crate::derive_utils::TryFromPyCell<'p, Self>;
    type Inst: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyDescrDeleteProtocol<'p>: PyDescrProtocol<'p> {
    type Inst: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyDescrSetNameProtocol<'p>: PyDescrProtocol<'p> {
    type Inst: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

/// All FFI functions for description protocols.
#[derive(Default)]
pub struct PyDescrMethods {
    pub tp_descr_get: Option<ffi::descrgetfunc>,
    pub tp_descr_set: Option<ffi::descrsetfunc>,
}

#[doc(hidden)]
impl PyDescrMethods {
    pub(crate) fn update_slots(&self, slots: &mut Vec<ffi::PyType_Slot>) {
        maybe_push_slot(
            slots,
            ffi::Py_tp_descr_get,
            self.tp_descr_get.map(|v| v as *mut c_void),
        );
        maybe_push_slot(
            slots,
            ffi::Py_tp_descr_set,
            self.tp_descr_set.map(|v| v as *mut c_void),
        );
    }
    pub fn set_descr_get<T>(&mut self)
    where
        T: for<'p> PyDescrGetProtocol<'p>,
    {
        self.tp_descr_get = py_ternarys_func!(PyDescrGetProtocol, T::__get__);
    }
    pub fn set_descr_set<T>(&mut self)
    where
        T: for<'p> PyDescrSetProtocol<'p>,
    {
        self.tp_descr_set = py_ternarys_func!(PyDescrSetProtocol, T::__set__, c_int);
    }
}
