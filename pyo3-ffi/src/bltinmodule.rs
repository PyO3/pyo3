use crate::object::PyTypeObject;

extern_python_dll! {
    pub static mut PyFilter_Type: PyTypeObject;
    pub static mut PyMap_Type: PyTypeObject;
    pub static mut PyZip_Type: PyTypeObject;
}
