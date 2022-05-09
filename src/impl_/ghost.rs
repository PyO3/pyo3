/// If it does nothing, was it ever really there? 👻
///
/// This is code that is just type checked to e.g. create better compile errors,
/// but that never affects anything at runtime,
use crate::{IntoPy, PyErr, PyObject};

pub trait IntoPyResult<T> {
    fn assert_into_py_result(&mut self) {}
}

impl<T> IntoPyResult<T> for T where T: IntoPy<PyObject> {}

impl<T, E> IntoPyResult<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
    E: Into<PyErr>,
{
}
