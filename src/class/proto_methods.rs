use crate::ffi;
#[cfg(not(Py_LIMITED_API))]
use crate::ffi::PyBufferProcs;

/// ABI3 doesn't have buffer APIs, so here we define the empty one.
#[cfg(Py_LIMITED_API)]
#[doc(hidden)]
#[derive(Clone)]
pub struct PyBufferProcs;

// Note(kngwyu): default implementations are for rust-numpy. Please don't remove them.
pub trait PyProtoMethods {
    fn get_type_slots() -> Vec<ffi::PyType_Slot> {
        vec![]
    }
    fn get_buffer() -> Option<PyBufferProcs> {
        None
    }
}

/// Typed version of `ffi::PyType_Slot`
#[doc(hidden)]
pub struct TypedSlot<T: Sized>(pub std::os::raw::c_int, pub T);

#[doc(hidden)]
pub enum PyProtoMethodDef {
    Slots(Vec<ffi::PyType_Slot>),
    Buffer(PyBufferProcs),
}

impl From<Vec<ffi::PyType_Slot>> for PyProtoMethodDef {
    fn from(slots: Vec<ffi::PyType_Slot>) -> Self {
        PyProtoMethodDef::Slots(slots)
    }
}

impl From<PyBufferProcs> for PyProtoMethodDef {
    fn from(buffer_procs: PyBufferProcs) -> Self {
        PyProtoMethodDef::Buffer(buffer_procs)
    }
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait PyProtoInventory: inventory::Collect {
    fn new(methods: PyProtoMethodDef) -> Self;
    fn get(&'static self) -> &'static PyProtoMethodDef;
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait HasProtoInventory {
    type ProtoMethods: PyProtoInventory;
}

#[cfg(feature = "macros")]
impl<T: HasProtoInventory> PyProtoMethods for T {
    fn get_type_slots() -> Vec<ffi::PyType_Slot> {
        inventory::iter::<T::ProtoMethods>
            .into_iter()
            .filter_map(|def| match def.get() {
                PyProtoMethodDef::Slots(slots) => Some(slots),
                PyProtoMethodDef::Buffer(_) => None,
            })
            .flatten()
            .cloned()
            .collect()
    }

    fn get_buffer() -> Option<PyBufferProcs> {
        inventory::iter::<T::ProtoMethods>
            .into_iter()
            .find_map(|def| match def.get() {
                PyProtoMethodDef::Slots(_) => None,
                PyProtoMethodDef::Buffer(buf) => Some(buf.clone()),
            })
    }
}
