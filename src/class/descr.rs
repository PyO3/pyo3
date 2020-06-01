// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Description Interface
//!
//! [Python information](
//! https://docs.python.org/3/reference/datamodel.html#implementing-descriptors)

use crate::err::PyResult;
use crate::types::{PyAny, PyType};
use crate::{ffi, FromPyObject, IntoPy, PyClass, PyObject};
use std::os::raw::c_int;

/// Descriptor interface
#[allow(unused_variables)]
pub trait PyDescrProtocol<'p>: PyClass {
    fn __get__(&'p self, instance: &'p PyAny, owner: Option<&'p PyType>) -> Self::Result
    where
        Self: PyDescrGetProtocol<'p>,
    {
        unimplemented!()
    }

    fn __set__(&'p self, instance: &'p PyAny, value: &'p PyAny) -> Self::Result
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
    type Inst: FromPyObject<'p>;
    type Owner: FromPyObject<'p>;
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyDescrSetProtocol<'p>: PyDescrProtocol<'p> {
    type Inst: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PyDescrDeleteProtocol<'p>: PyDescrProtocol<'p> {
    type Inst: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PyDescrSetNameProtocol<'p>: PyDescrProtocol<'p> {
    type Inst: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

#[derive(Default)]
pub struct PyDescrMethods {
    pub tp_descr_get: Option<ffi::descrgetfunc>,
    pub tp_descr_set: Option<ffi::descrsetfunc>,
}

impl PyDescrMethods {
    pub(crate) fn prepare_type_obj(&self, type_object: &mut ffi::PyTypeObject) {
        type_object.tp_descr_get = self.tp_descr_get;
        type_object.tp_descr_set = self.tp_descr_set;
    }
    pub fn set_descr_get<T>(&mut self)
    where
        T: for<'p> PyDescrGetProtocol<'p>,
    {
        self.tp_descr_get = py_ternary_func!(PyDescrGetProtocol, T::__get__);
    }
    pub fn set_descr_set<T>(&mut self)
    where
        T: for<'p> PyDescrSetProtocol<'p>,
    {
        self.tp_descr_set = py_ternary_func!(PyDescrSetProtocol, T::__set__, c_int);
    }
}
