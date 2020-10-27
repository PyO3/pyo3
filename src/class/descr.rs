// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! [Python information](
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors)

use super::proto_methods::TypedSlot;
use crate::callback::IntoPyCallbackOutput;
use crate::types::PyAny;
use crate::{ffi, FromPyObject, PyClass, PyObject};
use std::os::raw::c_int;

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

/// Extension trait for our proc-macro backend.
#[doc(hidden)]
pub trait PyDescrSlots {
    fn get_descr_get() -> TypedSlot<ffi::descrgetfunc>
    where
        Self: for<'p> PyDescrGetProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_tp_descr_get,
            py_ternarys_func!(PyDescrGetProtocol, Self::__get__),
        )
    }

    fn get_descr_set() -> TypedSlot<ffi::descrsetfunc>
    where
        Self: for<'p> PyDescrSetProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_tp_descr_set,
            py_ternarys_func!(PyDescrSetProtocol, Self::__set__, c_int),
        )
    }
}

impl<'p, T> PyDescrSlots for T where T: PyDescrProtocol<'p> {}
