use crate::class::buffer::PyBufferProcs;
use crate::ffi;

// Note(kngwyu): default implementations are for rust-numpy. Please don't remove them.
pub trait PyProtoMethods {
    fn get_type_slots() -> Vec<ffi::PyType_Slot> {
        vec![]
    }
    fn get_buffer() -> Option<PyBufferProcs> {
        None
    }
}

#[doc(hidden)]
pub enum PyProtoMethodDef {
    Slots(Vec<ffi::PyType_Slot>),
    Buffer(PyBufferProcs),
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait PyProtoMethodsInventory: inventory::Collect {
    fn new(methods: PyProtoMethodDef) -> Self;
    fn get(&'static self) -> &'static PyProtoMethodDef;
}

#[doc(hidden)]
#[cfg(feature = "macros")]
pub trait HasProtoMethodsInventory {
    type ProtoMethods: PyProtoMethodsInventory;
}

#[cfg(feature = "macros")]
impl<T: HasProtoMethodsInventory> PyProtoMethods for T {
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
