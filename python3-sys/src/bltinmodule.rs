use object::PyTypeObject;

extern "C" {
    pub static mut PyFilter_Type: PyTypeObject;
    pub static mut PyMap_Type: PyTypeObject;
    pub static mut PyZip_Type: PyTypeObject;
}
