use crate::ffi;
use std::marker::PhantomData;

// Note(kngwyu): default implementations are for rust-numpy. Please don't remove them.
pub trait PyProtoMethods {
    fn for_each_proto_slot<Visitor: FnMut(ffi::PyType_Slot)>(_visitor: Visitor) {}
    fn get_buffer() -> Option<&'static PyBufferProcs> {
        None
    }
}

pub struct PyClassProtocols<T>(PhantomData<T>);

impl<T> PyClassProtocols<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for PyClassProtocols<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PyClassProtocols<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Copy for PyClassProtocols<T> {}

// All traits describing slots, as well as the fallback implementations for unimplemented protos
//
// Protos which are implented use dtolnay specialization to implement for PyClassProtocols<T>.
//
// See https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md

pub trait PyObjectProtocolSlots<T> {
    fn object_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyObjectProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn object_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyDescrProtocolSlots<T> {
    fn descr_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyDescrProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn descr_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyGCProtocolSlots<T> {
    fn gc_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyGCProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn gc_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyIterProtocolSlots<T> {
    fn iter_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyIterProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn iter_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyMappingProtocolSlots<T> {
    fn mapping_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyMappingProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn mapping_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyNumberProtocolSlots<T> {
    fn number_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyNumberProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn number_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyAsyncProtocolSlots<T> {
    fn async_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyAsyncProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn async_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PySequenceProtocolSlots<T> {
    fn sequence_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PySequenceProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn sequence_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

pub trait PyBufferProtocolSlots<T> {
    fn buffer_protocol_slots(self) -> &'static [ffi::PyType_Slot];
}

impl<T> PyBufferProtocolSlots<T> for &'_ PyClassProtocols<T> {
    fn buffer_protocol_slots(self) -> &'static [ffi::PyType_Slot] {
        &[]
    }
}

// On Python < 3.9 setting the buffer protocol using slots doesn't work, so these procs are used
// on those versions to set the slots manually (on the limited API).

#[cfg(not(Py_LIMITED_API))]
pub use ffi::PyBufferProcs;

#[cfg(Py_LIMITED_API)]
pub struct PyBufferProcs;

pub trait PyBufferProtocolProcs<T> {
    fn buffer_procs(self) -> Option<&'static PyBufferProcs>;
}

impl<T> PyBufferProtocolProcs<T> for &'_ PyClassProtocols<T> {
    fn buffer_procs(self) -> Option<&'static PyBufferProcs> {
        None
    }
}
