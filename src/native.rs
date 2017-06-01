// Copyright (c) 2017-present PyO3 Project and Contributors

use objects::{PyObject};

pub trait PyBaseObject : Sized {}

pub trait PyNativeObject<'p> : PyBaseObject {

    fn as_object(self) -> PyObject<'p>;

    fn clone_object(&self) -> Self;

}
