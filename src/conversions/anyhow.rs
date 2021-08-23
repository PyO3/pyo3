use crate::PyErr;
use crate::exceptions::PyRuntimeError;

impl From<anyhow::Error> for PyErr {
    fn from(err: anyhow::Error) -> Self {
        PyRuntimeError::new_err(format!("{:?}", err))
    }
}
