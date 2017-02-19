
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


// !!!!!!!!!!!!!!!!!!!!!!!!!!!
// THIS IS A GENERATED FILE !!
//       DO NOT MODIFY      !!
// !!!!!!!!!!!!!!!!!!!!!!!!!!!

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_impl {
    // TT muncher macro. Results are accumulated in $info $slots $impls and $members.


    // Base case: we're done munching and can start producing code:
    {   {}
        $class:ident $py:ident
        /* info: */ {
            $base_type:ty,
            $size:expr,
            { $( $class_visibility:tt )* },
            $gc:tt,
            /* data: */ [ $( { $data_offset:expr, $data_name:ident, $data_ty:ty } )* ]
        }
        $slots:tt { $( $imp:item )* } $members:tt
    } => {
        py_coerce_item! {
            $($class_visibility)* struct $class { _unsafe_inner: $crate::PyObject }
        }

        py_impl_to_py_object_for_python_object!($class);
        py_impl_from_py_object_for_python_object!($class);

        impl $crate::PythonObject for $class {
            #[inline]
            fn as_object(&self) -> &$crate::PyObject {
                &self._unsafe_inner
            }

            #[inline]
            fn into_object(self) -> $crate::PyObject {
                self._unsafe_inner
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_from(obj: $crate::PyObject) -> Self {
                $class { _unsafe_inner: obj }
            }

            /// Unchecked downcast from PyObject to Self.
            /// Undefined behavior if the input object does not have the expected type.
            #[inline]
            unsafe fn unchecked_downcast_borrow_from<'a>(obj: &'a $crate::PyObject) -> &'a Self {
                ::std::mem::transmute(obj)
            }
        }

        impl $crate::PythonObjectWithCheckedDowncast for $class {
            #[inline]
            fn downcast_from<'p>(py: $crate::Python<'p>, obj: $crate::PyObject) -> Result<$class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, &obj) {
                    Ok($class { _unsafe_inner: obj })
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: $crate::Python<'p>, obj: &'a $crate::PyObject) -> Result<&'a $class, $crate::PythonObjectDowncastError<'p>> {
                if py.get_type::<$class>().is_instance(py, obj) {
                    unsafe { Ok(::std::mem::transmute(obj)) }
                } else {
                    Err($crate::PythonObjectDowncastError(py))
                }
            }
        }

        py_coerce_item! {
            impl $crate::py_class::BaseObject for $class {
                type InitType = ( $( $data_ty, )* );

                #[inline]
                fn size() -> usize {
                    $size
                }

                unsafe fn alloc(
                    py: $crate::Python,
                    ty: &$crate::PyType,
                    ( $( $data_name, )* ): Self::InitType
                ) -> $crate::PyResult<$crate::PyObject>
                {
                    let obj = try!(<$base_type as $crate::py_class::BaseObject>::alloc(py, ty, ()));
                    $( $crate::py_class::data_init::<$data_ty>(py, &obj, $data_offset, $data_name); )*
                    Ok(obj)
                }

                unsafe fn dealloc(py: $crate::Python, obj: *mut $crate::_detail::ffi::PyObject) {
                    $( $crate::py_class::data_drop::<$data_ty>(py, obj, $data_offset); )*
                    <$base_type as $crate::py_class::BaseObject>::dealloc(py, obj)
                }
            }
        }
        $($imp)*
        py_coerce_item! {
            impl $class {
                fn create_instance(py: $crate::Python $( , $data_name : $data_ty )* ) -> $crate::PyResult<$class> {
                    let obj = try!(unsafe {
                        <$class as $crate::py_class::BaseObject>::alloc(
                            py, &py.get_type::<$class>(), ( $($data_name,)* )
                        )
                    });
                    return Ok($class { _unsafe_inner: obj });

                    // hide statics in create_instance to avoid name conflicts
                    static mut TYPE_OBJECT : $crate::_detail::ffi::PyTypeObject
                        = py_class_type_object_static_init!($class, $gc, $slots);
                    static mut INIT_ACTIVE: bool = false;

                    // trait implementations that need direct access to TYPE_OBJECT
                    impl $crate::PythonObjectWithTypeObject for $class {
                        fn type_object(py: $crate::Python) -> $crate::PyType {
                            unsafe {
                                if $crate::py_class::is_ready(py, &TYPE_OBJECT) {
                                    $crate::PyType::from_type_ptr(py, &mut TYPE_OBJECT)
                                } else {
                                    // automatically initialize the class on-demand
                                    <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py, None)
                                        .expect(concat!("An error occurred while initializing class ", stringify!($class)))
                                }
                            }
                        }
                    }

                    impl $crate::py_class::PythonObjectFromPyClassMacro for $class {
                        fn initialize(py: $crate::Python, module_name: Option<&str>) -> $crate::PyResult<$crate::PyType> {
                            unsafe {
                                if $crate::py_class::is_ready(py, &TYPE_OBJECT) {
                                    return Ok($crate::PyType::from_type_ptr(py, &mut TYPE_OBJECT));
                                }
                                assert!(!INIT_ACTIVE,
                                    concat!("Reentrancy detected: already initializing class ",
                                    stringify!($class)));
                                INIT_ACTIVE = true;
                                let res = init(py, module_name);
                                INIT_ACTIVE = false;
                                res
                            }
                        }

                        fn add_to_module(py: $crate::Python, module: &$crate::PyModule) -> $crate::PyResult<()> {
                            let ty = <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py, module.name(py).ok())?;
                            module.add(py, stringify!($class), ty)
                        }
                    }

                    fn init($py: $crate::Python, module_name: Option<&str>) -> $crate::PyResult<$crate::PyType> {
                        py_class_type_object_dynamic_init!($class, $py, TYPE_OBJECT, module_name, $slots);
                        py_class_init_members!($class, $py, TYPE_OBJECT, $members);
                        unsafe {
                            if $crate::_detail::ffi::PyType_Ready(&mut TYPE_OBJECT) == 0 {
                                Ok($crate::PyType::from_type_ptr($py, &mut TYPE_OBJECT))
                            } else {
                                Err($crate::PyErr::fetch($py))
                            }
                        }
                    }
                }
            }
        }
    };

    { { data $data_name:ident : $data_type:ty; $($tail:tt)* }
        $class:ident $py:ident
        /* info: */ {
            $base_type: ty,
            $size: expr,
            $class_visibility: tt,
            $gc: tt,
            [ $( $data:tt )* ]
        }
        $slots:tt
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py
        /* info: */ {
            $base_type,
            /* size: */ $crate::py_class::data_new_size::<$data_type>($size),
            $class_visibility,
            $gc,
            /* data: */ [
                $($data)*
                {
                    $crate::py_class::data_offset::<$data_type>($size),
                    $data_name,
                    $data_type
                }
            ]
        }
        $slots
        /* impl: */ {
            $($imp)*
            impl $class {
                fn $data_name<'a>(&'a self, py: $crate::Python<'a>) -> &'a $data_type {
                    unsafe {
                        $crate::py_class::data_get::<$data_type>(
                        py,
                        &self._unsafe_inner,
                        $crate::py_class::data_offset::<$data_type>($size)
                        )
                    }
                }
            }
        }
        $members
    }};
    { { def __traverse__(&$slf:tt, $visit:ident) $body:block $($tail:tt)* }
        $class:ident $py:ident
        /* info: */ {
            $base_type: ty,
            $size: expr,
            $class_visibility: tt,
            /* gc: */ {
                /* traverse_proc: */ None,
                $traverse_data: tt
            },
            $datas: tt
        }
        $slots:tt
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py
        /* info: */ {
            $base_type,
            $size,
            $class_visibility,
            /* gc: */ {
                /* traverse_proc: */ $class::__traverse__,
                $traverse_data
            },
            $datas
        }
        $slots
        /* impl: */ {
            $($imp)*
            py_coerce_item!{
                impl $class {
                    fn __traverse__(&$slf,
                    $py: $crate::Python,
                    $visit: $crate::py_class::gc::VisitProc)
                    -> Result<(), $crate::py_class::gc::TraverseError>
                    $body
                }
            }
        }
        $members
    }};
    { { def __clear__ (&$slf:ident) $body:block $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_clear: py_class_tp_clear!($class),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_coerce_item!{
                impl $class {
                    fn __clear__(&$slf, $py: $crate::Python) $body
                }
            }
        }
        $members
    }};
    { { def __abs__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_absolute: py_class_unary_slot!($class::__abs__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __abs__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __abs__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __abs__" }
    };
    { { def __add__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_add: py_class_binary_numeric_slot!($class::__add__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __add__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __add__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __add__" }
    };

    { { def __aenter__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__aenter__ is not supported by py_class! yet." }
    };

    { { def __aexit__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__aexit__ is not supported by py_class! yet." }
    };

    { { def __aiter__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__aiter__ is not supported by py_class! yet." }
    };
    { { def __and__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_and: py_class_binary_numeric_slot!($class::__and__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __and__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __and__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __and__" }
    };

    { { def __await__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__await__ is not supported by py_class! yet." }
    };
    { { def __bool__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_nonzero: py_class_unary_slot!($class::__bool__, $crate::_detail::libc::c_int, $crate::py_class::slots::BoolConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __bool__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __bool__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __bool__" }
    };
    { {  def __call__ (&$slf:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_call: py_class_call_slot!{$class::__call__ []},
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __call__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};
    { {  def __call__ (&$slf:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_call: py_argparse_parse_plist_impl!{py_class_call_slot {$class::__call__} [] ($($p)+,)},
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, __call__(&$slf,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        $members
    }};

    { { def __cmp__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__cmp__ is not supported by py_class! use __richcmp__ instead." }
    };

    { { def __coerce__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__coerce__ is not supported by py_class! yet." }
    };

    { { def __complex__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__complex__ is not supported by py_class! yet." }
    };
    { { def __contains__(&$slf:ident, $item:ident : $item_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt
            /* as_sequence */ [ $( $sq_slot_name:ident : $sq_slot_value:expr, )* ]
            $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots $as_number
            /* as_sequence */ [
                $( $sq_slot_name : $sq_slot_value, )*
                sq_contains: py_class_contains_slot!($class::__contains__, $item_type),
            ]
            $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __contains__(&$slf,) $res_type; { $($body)* } [{ $item : $item_type = {} }] }
        }
        $members
    }};

    { { def __contains__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __contains__" }
    };

    { { def __del__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__del__ is not supported by py_class!; Use a data member with a Drop impl instead." }
    };

    { { def __delattr__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__delattr__ is not supported by py_class! yet." }
    };

    { { def __delete__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__delete__ is not supported by py_class! yet." }
    };
    { { def __delitem__(&$slf:ident, $key:ident : $key_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt $as_sequence:tt $as_mapping:tt
            /* setdelitem */ [
                sdi_setitem: $sdi_setitem_slot_value:tt,
                sdi_delitem: {},
            ]
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots $as_number $as_sequence $as_mapping
            /* setdelitem */ [
                sdi_setitem: $sdi_setitem_slot_value,
                sdi_delitem: { py_class_binary_slot!($class::__delitem__, $key_type, $crate::_detail::libc::c_int, $crate::py_class::slots::UnitCallbackConverter) },
            ]
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __delitem__(&$slf,) $res_type; { $($body)* } [{ $key : $key_type = {} }] }
        }
        $members
    }};

    { { def __delitem__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __delitem__" }
    };

    { { def __dir__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__dir__ is not supported by py_class! yet." }
    };

    { { def __div__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__div__ is not supported by py_class! yet." }
    };

    { { def __divmod__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__divmod__ is not supported by py_class! yet." }
    };

    { { def __eq__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__eq__ is not supported by py_class! use __richcmp__ instead." }
    };

    { { def __float__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__float__ is not supported by py_class! yet." }
    };

    { { def __floordiv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__floordiv__ is not supported by py_class! yet." }
    };

    { { def __ge__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__ge__ is not supported by py_class! use __richcmp__ instead." }
    };

    { { def __get__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__get__ is not supported by py_class! yet." }
    };

    { { def __getattr__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__getattr__ is not supported by py_class! yet." }
    };

    { { def __getattribute__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__getattribute__ is not supported by py_class! yet." }
    };
    { { def __getitem__(&$slf:ident, $key:ident : $key_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt
            /* as_sequence */ [ $( $sq_slot_name:ident : $sq_slot_value:expr, )* ]
            /* as_mapping */ [ $( $mp_slot_name:ident : $mp_slot_value:expr, )* ]
            $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots $as_number
            /* as_sequence */ [
                $( $sq_slot_name : $sq_slot_value, )*
                sq_item: Some($crate::py_class::slots::sq_item),
            ]
            /* as_mapping */ [
                $( $mp_slot_name : $mp_slot_value, )*
                mp_subscript: py_class_binary_slot!($class::__getitem__, $key_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __getitem__(&$slf,) $res_type; { $($body)* } [{ $key : $key_type = {} }] }
        }
        $members
    }};

    { { def __getitem__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __getitem__" }
    };

    { { def __gt__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__gt__ is not supported by py_class! use __richcmp__ instead." }
    };
    { { def __hash__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_hash: py_class_unary_slot!($class::__hash__, $crate::Py_hash_t, $crate::py_class::slots::HashConverter),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __hash__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __hash__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __hash__" }
    };
    { { def __iadd__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_add: py_class_binary_slot!($class::__iadd__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __iadd__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __iadd__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __iadd__" }
    };
    { { def __iand__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_and: py_class_binary_slot!($class::__iand__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __iand__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __iand__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __iand__" }
    };

    { { def __idiv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__idiv__ is not supported by py_class! yet." }
    };
    { { def __ifloordiv__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_floor_divide: py_class_binary_slot!($class::__ifloordiv__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __ifloordiv__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __ifloordiv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __ifloordiv__" }
    };
    { { def __ilshift__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_lshift: py_class_binary_slot!($class::__ilshift__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __ilshift__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __ilshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __ilshift__" }
    };
    { { def __imatmul__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_matrix_multiply: py_class_binary_slot!($class::__imatmul__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __imatmul__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __imatmul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __imatmul__" }
    };
    { { def __imod__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_remainder: py_class_binary_slot!($class::__imod__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __imod__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __imod__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __imod__" }
    };
    { { def __imul__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_multiply: py_class_binary_slot!($class::__imul__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __imul__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __imul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __imul__" }
    };

    { { def __index__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__index__ is not supported by py_class! yet." }
    };

    { { def __init__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__init__ is not supported by py_class!; use __new__ instead." }
    };

    { { def __instancecheck__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__instancecheck__ is not supported by py_class! yet." }
    };

    { { def __int__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__int__ is not supported by py_class! yet." }
    };
    { { def __invert__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_invert: py_class_unary_slot!($class::__invert__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __invert__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __invert__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __invert__" }
    };
    { { def __ior__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_or: py_class_binary_slot!($class::__ior__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __ior__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __ior__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __ior__" }
    };

    { { def __ipow__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__ipow__ is not supported by py_class! yet." }
    };
    { { def __irshift__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_rshift: py_class_binary_slot!($class::__irshift__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __irshift__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __irshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __irshift__" }
    };
    { { def __isub__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_subtract: py_class_binary_slot!($class::__isub__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __isub__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __isub__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __isub__" }
    };
    { { def __iter__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_iter: py_class_unary_slot!($class::__iter__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __iter__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __iter__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __iter__" }
    };
    { { def __itruediv__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_true_divide: py_class_binary_slot!($class::__itruediv__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __itruediv__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __itruediv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __itruediv__" }
    };
    { { def __ixor__(&$slf:ident, $other:ident : $other_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_inplace_xor: py_class_binary_slot!($class::__ixor__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __ixor__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} }] }
        }
        $members
    }};

    { { def __ixor__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __ixor__" }
    };

    { { def __le__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__le__ is not supported by py_class! use __richcmp__ instead." }
    };
    { { def __len__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt
            /* as_sequence */ [ $( $sq_slot_name:ident : $sq_slot_value:expr, )* ]
            /* as_mapping */ [ $( $mp_slot_name:ident : $mp_slot_value:expr, )* ]
            $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots $as_number
            /* as_sequence */ [
                $( $sq_slot_name : $sq_slot_value, )*
                sq_length: py_class_unary_slot!($class::__len__, $crate::_detail::ffi::Py_ssize_t, $crate::py_class::slots::LenResultConverter),
            ]
            /* as_mapping */ [
                $( $mp_slot_name : $mp_slot_value, )*
                mp_length: Some($crate::_detail::ffi::PySequence_Size),
            ]
            $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __len__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __len__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __len__" }
    };

    { { def __long__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__long__ is not supported by py_class! yet." }
    };
    { { def __lshift__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_lshift: py_class_binary_numeric_slot!($class::__lshift__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __lshift__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __lshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __lshift__" }
    };

    { { def __lt__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__lt__ is not supported by py_class! use __richcmp__ instead." }
    };

    { { def __matmul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__matmul__ is not supported by py_class! yet." }
    };

    { { def __mod__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__mod__ is not supported by py_class! yet." }
    };
    { { def __mul__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_multiply: py_class_binary_numeric_slot!($class::__mul__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __mul__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __mul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __mul__" }
    };

    { { def __ne__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__ne__ is not supported by py_class! use __richcmp__ instead." }
    };
    { { def __neg__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_negative: py_class_unary_slot!($class::__neg__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __neg__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __neg__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __neg__" }
    };
    { {  def __new__ ($cls:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_new: py_class_wrap_newfunc!{$class::__new__ []},
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py,__new__($cls: &$crate::PyType,) $res_type; { $($body)* } [] }
        }
        $members
    }};
    { {  def __new__ ($cls:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_new: py_argparse_parse_plist_impl!{py_class_wrap_newfunc {$class::__new__} [] ($($p)+,)},
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, __new__($cls: &$crate::PyType,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        $members
    }};
    { { def __next__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_iternext: py_class_unary_slot!($class::__next__, *mut $crate::_detail::ffi::PyObject, $crate::py_class::slots::IterNextResultConverter),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __next__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __next__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __next__" }
    };

    { { def __nonzero__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__nonzero__ is not supported by py_class!; use the Python 3 spelling __bool__ instead." }
    };
    { { def __or__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_or: py_class_binary_numeric_slot!($class::__or__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __or__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __or__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __or__" }
    };
    { { def __pos__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_positive: py_class_unary_slot!($class::__pos__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __pos__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __pos__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __pos__" }
    };

    { { def __pow__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__pow__ is not supported by py_class! yet." }
    };

    { { def __radd__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __radd__ is not supported by py_class! Use __add__ instead!" }
    };

    { { def __rand__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rand__ is not supported by py_class! Use __and__ instead!" }
    };

    { { def __rdiv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rdiv__ is not supported by py_class! Use __div__ instead!" }
    };

    { { def __rdivmod__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rdivmod__ is not supported by py_class! Use __divmod__ instead!" }
    };
    { { def __repr__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_repr: py_class_unary_slot!($class::__repr__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PythonObjectCallbackConverter::<$crate::PyString>(::std::marker::PhantomData)),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __repr__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __repr__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __repr__" }
    };

    { { def __rfloordiv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rfloordiv__ is not supported by py_class! Use __floordiv__ instead!" }
    };
    { { def __richcmp__(&$slf:ident, $other:ident : $other_type:ty, $op:ident : $op_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_richcompare: py_class_richcompare_slot!($class::__richcmp__, $other_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __richcmp__(&$slf,) $res_type; { $($body)* } [{ $other : $other_type = {} } { $op : $op_type = {} }] }
        }
        $members
    }};

    { { def __richcmp__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __richcmp__" }
    };

    { { def __rlshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rlshift__ is not supported by py_class! Use __lshift__ instead!" }
    };

    { { def __rmatmul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rmatmul__ is not supported by py_class! Use __matmul__ instead!" }
    };

    { { def __rmod__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rmod__ is not supported by py_class! Use __mod__ instead!" }
    };

    { { def __rmul__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rmul__ is not supported by py_class! Use __mul__ instead!" }
    };

    { { def __ror__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __ror__ is not supported by py_class! Use __or__ instead!" }
    };

    { { def __round__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__round__ is not supported by py_class! yet." }
    };

    { { def __rpow__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rpow__ is not supported by py_class! Use __pow__ instead!" }
    };

    { { def __rrshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rrshift__ is not supported by py_class! Use __rshift__ instead!" }
    };
    { { def __rshift__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_rshift: py_class_binary_numeric_slot!($class::__rshift__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __rshift__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __rshift__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __rshift__" }
    };

    { { def __rsub__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rsub__ is not supported by py_class! Use __sub__ instead!" }
    };

    { { def __rtruediv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rtruediv__ is not supported by py_class! Use __truediv__ instead!" }
    };

    { { def __rxor__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Reflected numeric operator __rxor__ is not supported by py_class! Use __xor__ instead!" }
    };

    { { def __set__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__set__ is not supported by py_class! yet." }
    };

    { { def __setattr__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__setattr__ is not supported by py_class! yet." }
    };
    { { def __setitem__(&$slf:ident, $key:ident : $key_type:ty, $value:ident : $value_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt $as_sequence:tt $as_mapping:tt
            /* setdelitem */ [
                sdi_setitem: {},
                sdi_delitem: $sdi_delitem_slot_value:tt,
            ]
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots $as_number $as_sequence $as_mapping
            /* setdelitem */ [
                sdi_setitem: { py_class_ternary_slot!($class::__setitem__, $key_type, $value_type, $crate::_detail::libc::c_int, $crate::py_class::slots::UnitCallbackConverter) },
                sdi_delitem: $sdi_delitem_slot_value,
            ]
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __setitem__(&$slf,) $res_type; { $($body)* } [{ $key : $key_type = {} } { $value : $value_type = {} }] }
        }
        $members
    }};

    { { def __setitem__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __setitem__" }
    };
    { { def __str__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_str: py_class_unary_slot!($class::__str__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PythonObjectCallbackConverter::<$crate::PyString>(::std::marker::PhantomData)),
            ]
            $as_number $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __str__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members
    }};

    { { def __str__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for operator __str__" }
    };
    { { def __sub__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_subtract: py_class_binary_numeric_slot!($class::__sub__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __sub__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __sub__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __sub__" }
    };

    { { def __subclasscheck__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__subclasscheck__ is not supported by py_class! yet." }
    };

    { { def __truediv__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "__truediv__ is not supported by py_class! yet." }
    };
    { { def __xor__($left:ident, $right:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt $setdelitem:tt
        }
        { $( $imp:item )* }
        $members:tt
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_xor: py_class_binary_numeric_slot!($class::__xor__),
            ]
            $as_sequence $as_mapping $setdelitem
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __xor__() $res_type; { $($body)* } [ { $left : &$crate::PyObject = {} } { $right : &$crate::PyObject = {} } ] }
        }
        $members
    }};

    { { def __xor__ $($tail:tt)* } $( $stuff:tt )* } => {
        py_error! { "Invalid signature for binary numeric operator __xor__" }
    };
    { {  def $name:ident (&$slf:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, $name(&$slf,) $res_type; { $($body)* } [] }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_class_instance_method!{$py, $class::$name []};
        }
    }};
    { {  def $name:ident (&$slf:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, $name(&$slf,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_argparse_parse_plist_impl!{py_class_instance_method {$py, $class::$name} [] ($($p)+,)};
        }
    }};
    { { @classmethod def $name:ident ($cls:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py,$name($cls: &$crate::PyType,) $res_type; { $($body)* } [] }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_class_class_method!{$py, $class::$name []};
        }
    }};
    { { @classmethod def $name:ident ($cls:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, $name($cls: &$crate::PyType,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_argparse_parse_plist_impl!{py_class_class_method {$py, $class::$name} [] ($($p)+,)};
        }
    }};
    { { @staticmethod def $name:ident ($($p:tt)*) -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist!{
                py_class_impl_item { $class, $py, $name() $res_type; { $($body)* } }
                ($($p)*)
            }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = 
            py_argparse_parse_plist!{
                py_class_static_method {$py, $class::$name}
                ($($p)*)
            }
            ;
        }
    }};
    { { static $name:ident = $init:expr; $($tail:tt)* }
        $class:ident $py:ident $info:tt $slots:tt $impls:tt
        { $( $member_name:ident = $member_expr:expr; )* }
    } => { py_class_impl! {
        { $($tail)* }
        $class $py $info $slots $impls
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = $init;
        }
    }};

}

