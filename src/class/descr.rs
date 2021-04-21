// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! [Python information](
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors)

use crate::callback::IntoPyCallbackOutput;
use crate::types::PyAny;
use crate::{FromPyObject, PyClass, PyObject};
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

    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__delete__` in `#[pymethods]` instead of in a protocol"
    )]
    fn __delete__(&'p self, instance: &'p PyAny) -> Self::Result
    where
        Self: PyDescrDeleteProtocol<'p>,
    {
        unimplemented!()
    }

    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__set_name__` in `#[pymethods]` instead of in a protocol"
    )]
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

py_ternarys_func!(descr_get, PyDescrGetProtocol, Self::__get__);
py_ternarys_func!(descr_set, PyDescrSetProtocol, Self::__set__, c_int);
