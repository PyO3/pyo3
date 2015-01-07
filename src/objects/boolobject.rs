use python::{Python, ToPythonPointer};

pyobject_newtype!(PyBool, PyBool_Check, PyBool_Type);

impl <'p> PyBool<'p> {
    #[inline]
    pub fn get(py: Python<'p>, val: bool) -> PyBool<'p> {
        if val { py.True() } else { py.False() }
    }
    
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { ::ffi::Py_True() }
    }
}

