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

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_static_init {
    ($name:ident, $size:expr,
    /* slots: */ {
        /* type_slots */  [ $( $slotname:ident : $slotvalue:expr, )* ]
        $as_number:tt
        $as_sequence:tt
    }) => (
        $crate::_detail::ffi::PyTypeObject {
            $( $slotname : $slotvalue, )*
            ..
            $crate::_detail::ffi::PyTypeObject_INIT
        }
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_type_object_dynamic_init {
    // initialize those fields of PyTypeObject that we couldn't initialize statically
    ($py:ident, $type_object:ident, $name: ident, $size: expr) => {{
        $type_object.tp_name = concat!(stringify!($name), "\0").as_ptr() as *const _;
        $type_object.tp_basicsize = $size as $crate::_detail::ffi::Py_ssize_t;
    }}
}

use ffi;



