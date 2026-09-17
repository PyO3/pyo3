use crate::object::PyTypeObject;

extern_libpython! {
    pub static mut PyEnum_Type: PyTypeObject;
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPyReversed_Type")]
    pub static mut PyReversed_Type: PyTypeObject;
}
