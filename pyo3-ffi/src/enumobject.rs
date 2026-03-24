use crate::object::PyTypeObject;

extern_libpython! {
    pub static mut PyEnum_Type: PyTypeObject;
    pub static mut PyReversed_Type: PyTypeObject;
}
