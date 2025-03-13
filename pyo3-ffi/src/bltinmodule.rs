use crate::object::PyTypeObject;

#[cfg_attr(windows, link(name = "pythonXY"))]
unsafe extern "C" {
    pub static mut PyFilter_Type: PyTypeObject;
    pub static mut PyMap_Type: PyTypeObject;
    pub static mut PyZip_Type: PyTypeObject;
}
