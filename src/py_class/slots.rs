// Copyright (c) 2016 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use ffi;
use std::mem;
use python::Python;

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_static_init {
    ($class_name:ident,
     $gc:tt,
    /* slots: */ {
        /* type_slots */  [ $( $slot_name:ident : $slot_value:expr, )* ]
        $as_number:tt
        $as_sequence:tt
    }) => (
        $crate::_detail::ffi::PyTypeObject {
            $( $slot_name : $slot_value, )*
            tp_dealloc: Some($crate::py_class::slots::tp_dealloc_callback::<$class_name>),
            tp_flags: py_class_type_object_flags!($gc),
            tp_traverse: py_class_tp_traverse!($class_name, $gc),
            ..
            $crate::_detail::ffi::PyTypeObject_INIT
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_flags {
    (/* gc: */ {
        /* traverse_proc: */ None,
        /* traverse_data: */ [ /*name*/ ]
    }) => {
        $crate::_detail::ffi::Py_TPFLAGS_DEFAULT
    };
    (/* gc: */ {
        $traverse_proc: expr,
        $traverse_data: tt
    }) => {
        $crate::_detail::ffi::Py_TPFLAGS_DEFAULT
        | $crate::_detail::ffi::Py_TPFLAGS_HAVE_GC
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_dynamic_init {
    // initialize those fields of PyTypeObject that we couldn't initialize statically
    ($class: ident, $py:ident, $type_object:ident) => {
        unsafe {
            $type_object.tp_name = concat!(stringify!($class), "\0").as_ptr() as *const _;
            $type_object.tp_basicsize = <$class as $crate::py_class::BaseObject>::size()
                                        as $crate::_detail::ffi::Py_ssize_t;
        }
    }
}

pub unsafe extern "C" fn tp_dealloc_callback<T>(obj: *mut ffi::PyObject)
    where T: super::BaseObject
{
    let guard = ::function::AbortOnDrop("Cannot unwind out of tp_dealloc");
    let py = Python::assume_gil_acquired();
    let r = T::dealloc(py, obj);
    mem::forget(guard);
    r
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_wrap_newfunc {
    ($class:ident :: $f:ident [ $( { $pname:ident : $ptype:ty = $detail:tt } )* ]) => {{
        unsafe extern "C" fn wrap_newfunc<DUMMY>(
            cls: *mut $crate::_detail::ffi::PyTypeObject,
            args: *mut $crate::_detail::ffi::PyObject,
            kwargs: *mut $crate::_detail::ffi::PyObject)
        -> *mut $crate::_detail::ffi::PyObject
        {
            const LOCATION: &'static str = concat!(stringify!($class), ".", stringify!($f), "()");
            $crate::_detail::handle_callback(
                LOCATION,
                |py| {
                    py_argparse_raw!(py, Some(LOCATION), args, kwargs,
                        [ $( { $pname : $ptype = $detail } )* ]
                        {
                            let cls = $crate::PyType::from_type_ptr(py, cls);
                            let ret = $class::$f(&cls, py $(, $pname )* );
                            $crate::PyDrop::release_ref(cls, py);
                            ret
                        })
                })
        }
        wrap_newfunc::<()>
    }}
}
