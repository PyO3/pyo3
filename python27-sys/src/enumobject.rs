use object::PyTypeObject;

#[link(name = "python2.7")]
extern "C" {
    pub static mut PyEnum_Type: PyTypeObject;
    pub static mut PyReversed_Type: PyTypeObject;
}

