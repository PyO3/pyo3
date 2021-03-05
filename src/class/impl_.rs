// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{derive_utils::PyBaseTypeUtils, ffi, PyMethodDefType, PyNativeType};
use std::{marker::PhantomData, thread};

/// This type is used as a "dummy" type on which dtolnay specializations are
/// applied to apply implementations from `#[pymethods]` & `#[pyproto]`
pub struct PyClassImplCollector<T>(PhantomData<T>);

impl<T> PyClassImplCollector<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for PyClassImplCollector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for PyClassImplCollector<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Copy for PyClassImplCollector<T> {}

/// Implements the underlying functionality of `#[pyclass]`, assembled by various proc macros.
///
/// Users are discouraged from implementing this trait manually; it is a PyO3 implementation detail
/// and may be changed at any time.
pub trait PyClassImpl: Sized {
    /// This handles following two situations:
    /// 1. In case `T` is `Send`, stub `ThreadChecker` is used and does nothing.
    ///    This implementation is used by default. Compile fails if `T: !Send`.
    /// 2. In case `T` is `!Send`, `ThreadChecker` panics when `T` is accessed by another thread.
    ///    This implementation is used when `#[pyclass(unsendable)]` is given.
    ///    Panicking makes it safe to expose `T: !Send` to the Python interpreter, where all objects
    ///    can be accessed by multiple threads by `threading` module.
    type ThreadChecker: PyClassThreadChecker<Self>;

    fn for_each_method_def(_visitor: impl FnMut(&PyMethodDefType)) {}
    fn get_new() -> Option<ffi::newfunc> {
        None
    }
    fn get_call() -> Option<ffi::PyCFunctionWithKeywords> {
        None
    }
    fn for_each_proto_slot(_visitor: impl FnMut(&ffi::PyType_Slot)) {}
    fn get_buffer() -> Option<&'static PyBufferProcs> {
        None
    }
}

// Traits describing known special methods.

pub trait PyClassNewImpl<T> {
    fn new_impl(self) -> Option<ffi::newfunc>;
}

impl<T> PyClassNewImpl<T> for &'_ PyClassImplCollector<T> {
    fn new_impl(self) -> Option<ffi::newfunc> {
        None
    }
}

pub trait PyClassCallImpl<T> {
    fn call_impl(self) -> Option<ffi::PyCFunctionWithKeywords>;
}

impl<T> PyClassCallImpl<T> for &'_ PyClassImplCollector<T> {
    fn call_impl(self) -> Option<ffi::PyCFunctionWithKeywords> {
        None
    }
}

// All traits describing slots, as well as the fallback implementations for unimplemented protos
//
// Protos which are implemented use dtolnay specialization to implement for PyClassImplCollector<T>.
//
// See https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md

macro_rules! slots_trait {
    ($name:ident, $function_name: ident) => {
        pub trait $name<T> {
            fn $function_name(self) -> &'static [ffi::PyType_Slot];
        }

        impl<T> $name<T> for &'_ PyClassImplCollector<T> {
            fn $function_name(self) -> &'static [ffi::PyType_Slot] {
                &[]
            }
        }
    };
}

slots_trait!(PyObjectProtocolSlots, object_protocol_slots);
slots_trait!(PyDescrProtocolSlots, descr_protocol_slots);
slots_trait!(PyGCProtocolSlots, gc_protocol_slots);
slots_trait!(PyIterProtocolSlots, iter_protocol_slots);
slots_trait!(PyMappingProtocolSlots, mapping_protocol_slots);
slots_trait!(PyNumberProtocolSlots, number_protocol_slots);
slots_trait!(PyAsyncProtocolSlots, async_protocol_slots);
slots_trait!(PySequenceProtocolSlots, sequence_protocol_slots);
slots_trait!(PyBufferProtocolSlots, buffer_protocol_slots);

macro_rules! methods_trait {
    ($name:ident, $function_name: ident) => {
        pub trait $name<T> {
            fn $function_name(self) -> &'static [PyMethodDefType];
        }

        impl<T> $name<T> for &'_ PyClassImplCollector<T> {
            fn $function_name(self) -> &'static [PyMethodDefType] {
                &[]
            }
        }
    };
}

methods_trait!(PyObjectProtocolMethods, object_protocol_methods);
methods_trait!(PyAsyncProtocolMethods, async_protocol_methods);
methods_trait!(PyContextProtocolMethods, context_protocol_methods);
methods_trait!(PyDescrProtocolMethods, descr_protocol_methods);
methods_trait!(PyMappingProtocolMethods, mapping_protocol_methods);
methods_trait!(PyNumberProtocolMethods, number_protocol_methods);

// On Python < 3.9 setting the buffer protocol using slots doesn't work, so these procs are used
// on those versions to set the slots manually (on the limited API).

#[cfg(not(Py_LIMITED_API))]
pub use ffi::PyBufferProcs;

#[cfg(Py_LIMITED_API)]
pub struct PyBufferProcs;

pub trait PyBufferProtocolProcs<T> {
    fn buffer_procs(self) -> Option<&'static PyBufferProcs>;
}

impl<T> PyBufferProtocolProcs<T> for &'_ PyClassImplCollector<T> {
    fn buffer_procs(self) -> Option<&'static PyBufferProcs> {
        None
    }
}

// Thread checkers

#[doc(hidden)]
pub trait PyClassThreadChecker<T>: Sized {
    fn ensure(&self);
    fn new() -> Self;
    private_decl! {}
}

/// Stub checker for `Send` types.
#[doc(hidden)]
pub struct ThreadCheckerStub<T: Send>(PhantomData<T>);

impl<T: Send> PyClassThreadChecker<T> for ThreadCheckerStub<T> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

impl<T: PyNativeType> PyClassThreadChecker<T> for ThreadCheckerStub<crate::PyObject> {
    fn ensure(&self) {}
    fn new() -> Self {
        ThreadCheckerStub(PhantomData)
    }
    private_impl! {}
}

/// Thread checker for unsendable types.
/// Panics when the value is accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerImpl<T>(thread::ThreadId, PhantomData<T>);

impl<T> PyClassThreadChecker<T> for ThreadCheckerImpl<T> {
    fn ensure(&self) {
        if thread::current().id() != self.0 {
            panic!(
                "{} is unsendable, but sent to another thread!",
                std::any::type_name::<T>()
            );
        }
    }
    fn new() -> Self {
        ThreadCheckerImpl(thread::current().id(), PhantomData)
    }
    private_impl! {}
}

/// Thread checker for types that have `Send` and `extends=...`.
/// Ensures that `T: Send` and the parent is not accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerInherited<T: Send, U: PyBaseTypeUtils>(PhantomData<T>, U::ThreadChecker);

impl<T: Send, U: PyBaseTypeUtils> PyClassThreadChecker<T> for ThreadCheckerInherited<T, U> {
    fn ensure(&self) {
        self.1.ensure();
    }
    fn new() -> Self {
        ThreadCheckerInherited(PhantomData, U::ThreadChecker::new())
    }
    private_impl! {}
}
