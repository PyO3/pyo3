use crate::object::PyTypeObject;

extern_python_dll! {
    pub static mut PyEnum_Type: PyTypeObject;
    pub static mut PyReversed_Type: PyTypeObject;
}
