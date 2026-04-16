use crate::object::*;

pub type PyCapsule_Destructor = unsafe extern "C" fn(o: *mut PyObject);

pub use crate::backend::current::pycapsule::{
    PyCapsule_CheckExact, PyCapsule_GetContext, PyCapsule_GetDestructor, PyCapsule_GetName,
    PyCapsule_GetPointer, PyCapsule_Import, PyCapsule_IsValid, PyCapsule_New, PyCapsule_SetContext,
    PyCapsule_SetDestructor, PyCapsule_SetName, PyCapsule_SetPointer, PyCapsule_Type,
};
