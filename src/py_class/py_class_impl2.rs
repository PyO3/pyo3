
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
    { $class:ident $py:ident
        /* info: */ {
            $base_type:ty,
            $size:expr,
            $gc:tt,
            /* data: */ [ $( { $data_offset:expr, $data_name:ident, $data_ty:ty } )* ]
        }
        $slots:tt { $( $imp:item )* } $members:tt;
    } => {
        struct $class { _unsafe_inner: $crate::PyObject }

        pyobject_to_pyobject!($class);

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
                    static mut type_object : $crate::_detail::ffi::PyTypeObject
                        = py_class_type_object_static_init!($class, $gc, $slots);
                    static mut init_active: bool = false;

                    // trait implementations that need direct access to type_object
                    impl $crate::PythonObjectWithTypeObject for $class {
                        fn type_object(py: $crate::Python) -> $crate::PyType {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    $crate::PyType::from_type_ptr(py, &mut type_object)
                                } else {
                                    // automatically initialize the class on-demand
                                    <$class as $crate::py_class::PythonObjectFromPyClassMacro>::initialize(py)
                                        .expect(concat!("An error occurred while initializing class ", stringify!($class)))
                                }
                            }
                        }
                    }

                    impl $crate::py_class::PythonObjectFromPyClassMacro for $class {
                        fn initialize(py: $crate::Python) -> $crate::PyResult<$crate::PyType> {
                            unsafe {
                                if $crate::py_class::is_ready(py, &type_object) {
                                    return Ok($crate::PyType::from_type_ptr(py, &mut type_object));
                                }
                                assert!(!init_active,
                                    concat!("Reentrancy detected: already initializing class ",
                                    stringify!($class)));
                                init_active = true;
                                let res = init(py);
                                init_active = false;
                                res
                            }
                        }
                    }

                    fn init($py: $crate::Python) -> $crate::PyResult<$crate::PyType> {
                        py_class_type_object_dynamic_init!($class, $py, type_object, $slots);
                        py_class_init_members!($class, $py, type_object, $members);
                        unsafe {
                            if $crate::_detail::ffi::PyType_Ready(&mut type_object) == 0 {
                                Ok($crate::PyType::from_type_ptr($py, &mut type_object))
                            } else {
                                Err($crate::PyErr::fetch($py))
                            }
                        }
                    }
                }
            }
        }
    };

    { $class:ident $py:ident
        /* info: */ {
            $base_type: ty,
            $size: expr,
            $gc: tt,
            [ $( $data:tt )* ]
        }
        $slots:tt
        { $( $imp:item )* }
        $members:tt;
        data $data_name:ident : $data_type:ty; $($tail:tt)*
    } => { py_class_impl! {
        $class $py
        /* info: */ {
            $base_type,
            /* size: */ $crate::py_class::data_new_size::<$data_type>($size),
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
        $members; $($tail)*
    }};
    { $class:ident $py:ident
        /* info: */ {
            $base_type: ty,
            $size: expr,
            /* gc: */ {
                /* traverse_proc: */ None,
                $traverse_data: tt
            },
            $datas: tt
        }
        $slots:tt
        { $( $imp:item )* }
        $members:tt;
        def __traverse__(&$slf:tt, $visit:ident) $body:block $($tail:tt)*
    } => { py_class_impl! {
        $class $py
        /* info: */ {
            $base_type,
            $size,
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
        $members; $($tail)*
    }};
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __clear__ (&$slf:ident) $body:block $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_clear: py_class_tp_clear!($class),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_coerce_item!{
                impl $class {
                    fn __clear__(&$slf, $py: $crate::Python) $body
                }
            }
        }
        $members; $($tail)*
    }};
// def __abs__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __abs__ $($tail:tt)*
    } => {
        py_error! { "__abs__ is not supported by py_class! yet." }
    };
// def __add__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __add__ $($tail:tt)*
    } => {
        py_error! { "__add__ is not supported by py_class! yet." }
    };
// def __aenter__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aenter__ $($tail:tt)*
    } => {
        py_error! { "__aenter__ is not supported by py_class! yet." }
    };
// def __aexit__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aexit__ $($tail:tt)*
    } => {
        py_error! { "__aexit__ is not supported by py_class! yet." }
    };
// def __aiter__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __aiter__ $($tail:tt)*
    } => {
        py_error! { "__aiter__ is not supported by py_class! yet." }
    };
// def __and__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __and__ $($tail:tt)*
    } => {
        py_error! { "__and__ is not supported by py_class! yet." }
    };
// def __await__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __await__ $($tail:tt)*
    } => {
        py_error! { "__await__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt
            /* as_number */ [ $( $nb_slot_name:ident : $nb_slot_value:expr, )* ]
            $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __bool__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            $type_slots
            /* as_number */ [
                $( $nb_slot_name : $nb_slot_value, )*
                nb_nonzero: py_class_unary_slot!($class::__bool__, $crate::_detail::libc::c_int, $crate::py_class::slots::BoolConverter),
            ]
            $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __bool__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __bool__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __bool__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __bool__" }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __call__ (&$slf:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_call: py_class_call_slot!{$class::__call__ []},
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __call__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __call__ (&$slf:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_call: py_argparse_parse_plist_impl!{py_class_call_slot {$class::__call__} [] ($($p)+,)},
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, __call__(&$slf,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        $members; $($tail)*
    }};
// def __cmp__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __cmp__ $($tail:tt)*
    } => {
        py_error! { "__cmp__ is not supported by py_class! yet." }
    };
// def __coerce__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __coerce__ $($tail:tt)*
    } => {
        py_error! { "__coerce__ is not supported by py_class! yet." }
    };
// def __complex__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __complex__ $($tail:tt)*
    } => {
        py_error! { "__complex__ is not supported by py_class! yet." }
    };
// def __contains__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __contains__ $($tail:tt)*
    } => {
        py_error! { "__contains__ is not supported by py_class! yet." }
    };
// def __del__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __del__ $($tail:tt)*
    } => {
        py_error! { "__del__ is not supported by py_class!; Use a data member with a Drop impl instead." }
    };
// def __delattr__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delattr__ $($tail:tt)*
    } => {
        py_error! { "__delattr__ is not supported by py_class! yet." }
    };
// def __delete__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delete__ $($tail:tt)*
    } => {
        py_error! { "__delete__ is not supported by py_class! yet." }
    };
// def __delitem__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __delitem__ $($tail:tt)*
    } => {
        py_error! { "__delitem__ is not supported by py_class! yet." }
    };
// def __dir__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __dir__ $($tail:tt)*
    } => {
        py_error! { "__dir__ is not supported by py_class! yet." }
    };
// def __div__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __div__ $($tail:tt)*
    } => {
        py_error! { "__div__ is not supported by py_class! yet." }
    };
// def __divmod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __divmod__ $($tail:tt)*
    } => {
        py_error! { "__divmod__ is not supported by py_class! yet." }
    };
// def __enter__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __enter__ $($tail:tt)*
    } => {
        py_error! { "__enter__ is not supported by py_class! yet." }
    };
// def __eq__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __eq__ $($tail:tt)*
    } => {
        py_error! { "__eq__ is not supported by py_class! yet." }
    };
// def __exit__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __exit__ $($tail:tt)*
    } => {
        py_error! { "__exit__ is not supported by py_class! yet." }
    };
// def __float__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __float__ $($tail:tt)*
    } => {
        py_error! { "__float__ is not supported by py_class! yet." }
    };
// def __floordiv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __floordiv__ $($tail:tt)*
    } => {
        py_error! { "__floordiv__ is not supported by py_class! yet." }
    };
// def __ge__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ge__ $($tail:tt)*
    } => {
        py_error! { "__ge__ is not supported by py_class! yet." }
    };
// def __get__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __get__ $($tail:tt)*
    } => {
        py_error! { "__get__ is not supported by py_class! yet." }
    };
// def __getattr__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getattr__ $($tail:tt)*
    } => {
        py_error! { "__getattr__ is not supported by py_class! yet." }
    };
// def __getattribute__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getattribute__ $($tail:tt)*
    } => {
        py_error! { "__getattribute__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt
            /* as_sequence */ [ $( $sq_slot_name:ident : $sq_slot_value:expr, )* ]
            /* as_mapping */ [ $( $mp_slot_name:ident : $mp_slot_value:expr, )* ]
        }
        { $( $imp:item )* }
        $members:tt;
        def __getitem__(&$slf:ident, $x:ident : $x_type:ty) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            $type_slots $as_number
            /* as_sequence */ [
                $( $sq_slot_name : $sq_slot_value, )*
                sq_item: Some($crate::py_class::slots::sq_item),
            ]
            /* as_mapping */ [
                $( $mp_slot_name : $mp_slot_value, )*
                mp_subscript: py_class_binary_slot!($class::__getitem__, $x_type, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __getitem__(&$slf,) $res_type; { $($body)* } [{ $x : $x_type = {} }] }
        }
        $members; $($tail)*
    }};
// def __getitem__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __getitem__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __getitem__" }
    };
// def __gt__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __gt__ $($tail:tt)*
    } => {
        py_error! { "__gt__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __hash__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_hash: py_class_unary_slot!($class::__hash__, $crate::Py_hash_t, $crate::py_class::slots::HashConverter),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __hash__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __hash__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __hash__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __hash__" }
    };
// def __iadd__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iadd__ $($tail:tt)*
    } => {
        py_error! { "__iadd__ is not supported by py_class! yet." }
    };
// def __iand__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iand__ $($tail:tt)*
    } => {
        py_error! { "__iand__ is not supported by py_class! yet." }
    };
// def __idiv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __idiv__ $($tail:tt)*
    } => {
        py_error! { "__idiv__ is not supported by py_class! yet." }
    };
// def __idivmod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __idivmod__ $($tail:tt)*
    } => {
        py_error! { "__idivmod__ is not supported by py_class! yet." }
    };
// def __ifloordiv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ifloordiv__ $($tail:tt)*
    } => {
        py_error! { "__ifloordiv__ is not supported by py_class! yet." }
    };
// def __ilshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ilshift__ $($tail:tt)*
    } => {
        py_error! { "__ilshift__ is not supported by py_class! yet." }
    };
// def __imatmul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imatmul__ $($tail:tt)*
    } => {
        py_error! { "__imatmul__ is not supported by py_class! yet." }
    };
// def __imod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imod__ $($tail:tt)*
    } => {
        py_error! { "__imod__ is not supported by py_class! yet." }
    };
// def __imul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __imul__ $($tail:tt)*
    } => {
        py_error! { "__imul__ is not supported by py_class! yet." }
    };
// def __index__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __index__ $($tail:tt)*
    } => {
        py_error! { "__index__ is not supported by py_class! yet." }
    };
// def __init__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __init__ $($tail:tt)*
    } => {
        py_error! { "__init__ is not supported by py_class!; use __new__ instead." }
    };
// def __instancecheck__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __instancecheck__ $($tail:tt)*
    } => {
        py_error! { "__instancecheck__ is not supported by py_class! yet." }
    };
// def __int__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __int__ $($tail:tt)*
    } => {
        py_error! { "__int__ is not supported by py_class! yet." }
    };
// def __invert__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __invert__ $($tail:tt)*
    } => {
        py_error! { "__invert__ is not supported by py_class! yet." }
    };
// def __ior__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ior__ $($tail:tt)*
    } => {
        py_error! { "__ior__ is not supported by py_class! yet." }
    };
// def __ipow__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ipow__ $($tail:tt)*
    } => {
        py_error! { "__ipow__ is not supported by py_class! yet." }
    };
// def __irshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __irshift__ $($tail:tt)*
    } => {
        py_error! { "__irshift__ is not supported by py_class! yet." }
    };
// def __isub__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __isub__ $($tail:tt)*
    } => {
        py_error! { "__isub__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __iter__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_iter: py_class_unary_slot!($class::__iter__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PyObjectCallbackConverter),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __iter__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __iter__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __iter__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __iter__" }
    };
// def __itruediv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __itruediv__ $($tail:tt)*
    } => {
        py_error! { "__itruediv__ is not supported by py_class! yet." }
    };
// def __ixor__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ixor__ $($tail:tt)*
    } => {
        py_error! { "__ixor__ is not supported by py_class! yet." }
    };
// def __le__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __le__ $($tail:tt)*
    } => {
        py_error! { "__le__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            $type_slots:tt $as_number:tt
            /* as_sequence */ [ $( $sq_slot_name:ident : $sq_slot_value:expr, )* ]
            /* as_mapping */ [ $( $mp_slot_name:ident : $mp_slot_value:expr, )* ]
        }
        { $( $imp:item )* }
        $members:tt;
        def __len__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
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
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __len__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __len__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __len__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __len__" }
    };
// def __long__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __long__ $($tail:tt)*
    } => {
        py_error! { "__long__ is not supported by py_class! yet." }
    };
// def __lshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __lshift__ $($tail:tt)*
    } => {
        py_error! { "__lshift__ is not supported by py_class! yet." }
    };
// def __lt__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __lt__ $($tail:tt)*
    } => {
        py_error! { "__lt__ is not supported by py_class! yet." }
    };
// def __matmul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __matmul__ $($tail:tt)*
    } => {
        py_error! { "__matmul__ is not supported by py_class! yet." }
    };
// def __mod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __mod__ $($tail:tt)*
    } => {
        py_error! { "__mod__ is not supported by py_class! yet." }
    };
// def __mul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __mul__ $($tail:tt)*
    } => {
        py_error! { "__mul__ is not supported by py_class! yet." }
    };
// def __ne__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ne__ $($tail:tt)*
    } => {
        py_error! { "__ne__ is not supported by py_class! yet." }
    };
// def __neg__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __neg__ $($tail:tt)*
    } => {
        py_error! { "__neg__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __new__ ($cls:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_new: py_class_wrap_newfunc!{$class::__new__ []},
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py,__new__($cls: &$crate::PyType,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __new__ ($cls:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_new: py_argparse_parse_plist_impl!{py_class_wrap_newfunc {$class::__new__} [] ($($p)+,)},
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_argparse_parse_plist_impl!{
                py_class_impl_item { $class, $py, __new__($cls: &$crate::PyType,) $res_type; { $($body)* } }
                [] ($($p)+,)
            }
        }
        $members; $($tail)*
    }};
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __next__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_iternext: py_class_unary_slot!($class::__next__, *mut $crate::_detail::ffi::PyObject, $crate::py_class::slots::IterNextResultConverter),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __next__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __next__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __next__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __next__" }
    };
// def __nonzero__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __nonzero__ $($tail:tt)*
    } => {
        py_error! { "__nonzero__ is not supported by py_class!; use the Python 3 spelling __bool__ instead." }
    };
// def __or__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __or__ $($tail:tt)*
    } => {
        py_error! { "__or__ is not supported by py_class! yet." }
    };
// def __pos__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __pos__ $($tail:tt)*
    } => {
        py_error! { "__pos__ is not supported by py_class! yet." }
    };
// def __pow__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __pow__ $($tail:tt)*
    } => {
        py_error! { "__pow__ is not supported by py_class! yet." }
    };
// def __radd__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __radd__ $($tail:tt)*
    } => {
        py_error! { "__radd__ is not supported by py_class! yet." }
    };
// def __rand__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rand__ $($tail:tt)*
    } => {
        py_error! { "__rand__ is not supported by py_class! yet." }
    };
// def __rdiv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rdiv__ $($tail:tt)*
    } => {
        py_error! { "__rdiv__ is not supported by py_class! yet." }
    };
// def __rdivmod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rdivmod__ $($tail:tt)*
    } => {
        py_error! { "__rdivmod__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __repr__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_repr: py_class_unary_slot!($class::__repr__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PythonObjectCallbackConverter::<$crate::PyString>(::std::marker::PhantomData)),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __repr__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __repr__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __repr__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __repr__" }
    };
// def __reversed__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __reversed__ $($tail:tt)*
    } => {
        py_error! { "__reversed__ is not supported by py_class! yet." }
    };
// def __rfloordiv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rfloordiv__ $($tail:tt)*
    } => {
        py_error! { "__rfloordiv__ is not supported by py_class! yet." }
    };
// def __rlshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rlshift__ $($tail:tt)*
    } => {
        py_error! { "__rlshift__ is not supported by py_class! yet." }
    };
// def __rmatmul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmatmul__ $($tail:tt)*
    } => {
        py_error! { "__rmatmul__ is not supported by py_class! yet." }
    };
// def __rmod__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmod__ $($tail:tt)*
    } => {
        py_error! { "__rmod__ is not supported by py_class! yet." }
    };
// def __rmul__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rmul__ $($tail:tt)*
    } => {
        py_error! { "__rmul__ is not supported by py_class! yet." }
    };
// def __ror__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __ror__ $($tail:tt)*
    } => {
        py_error! { "__ror__ is not supported by py_class! yet." }
    };
// def __round__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __round__ $($tail:tt)*
    } => {
        py_error! { "__round__ is not supported by py_class! yet." }
    };
// def __rpow__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rpow__ $($tail:tt)*
    } => {
        py_error! { "__rpow__ is not supported by py_class! yet." }
    };
// def __rrshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rrshift__ $($tail:tt)*
    } => {
        py_error! { "__rrshift__ is not supported by py_class! yet." }
    };
// def __rshift__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rshift__ $($tail:tt)*
    } => {
        py_error! { "__rshift__ is not supported by py_class! yet." }
    };
// def __rsub__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rsub__ $($tail:tt)*
    } => {
        py_error! { "__rsub__ is not supported by py_class! yet." }
    };
// def __rtruediv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rtruediv__ $($tail:tt)*
    } => {
        py_error! { "__rtruediv__ is not supported by py_class! yet." }
    };
// def __rxor__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __rxor__ $($tail:tt)*
    } => {
        py_error! { "__rxor__ is not supported by py_class! yet." }
    };
// def __set__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __set__ $($tail:tt)*
    } => {
        py_error! { "__set__ is not supported by py_class! yet." }
    };
// def __setattr__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __setattr__ $($tail:tt)*
    } => {
        py_error! { "__setattr__ is not supported by py_class! yet." }
    };
// def __setitem__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __setitem__ $($tail:tt)*
    } => {
        py_error! { "__setitem__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt
        /* slots: */ {
            /* type_slots */ [ $( $tp_slot_name:ident : $tp_slot_value:expr, )* ]
            $as_number:tt $as_sequence:tt $as_mapping:tt
        }
        { $( $imp:item )* }
        $members:tt;
        def __str__(&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info
        /* slots: */ {
            /* type_slots */ [
                $( $tp_slot_name : $tp_slot_value, )*
                tp_str: py_class_unary_slot!($class::__str__, *mut $crate::_detail::ffi::PyObject, $crate::_detail::PythonObjectCallbackConverter::<$crate::PyString>(::std::marker::PhantomData)),
            ]
            $as_number $as_sequence $as_mapping
        }
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, __str__(&$slf,) $res_type; { $($body)* } [] }
        }
        $members; $($tail)*
    }};
// def __str__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __str__ $($tail:tt)*
    } => {
        py_error! { "Invalid signature for unary operator __str__" }
    };
// def __sub__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __sub__ $($tail:tt)*
    } => {
        py_error! { "__sub__ is not supported by py_class! yet." }
    };
// def __subclasscheck__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __subclasscheck__ $($tail:tt)*
    } => {
        py_error! { "__subclasscheck__ is not supported by py_class! yet." }
    };
// def __truediv__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __truediv__ $($tail:tt)*
    } => {
        py_error! { "__truediv__ is not supported by py_class! yet." }
    };
// def __xor__()
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt $members:tt;
        def __xor__ $($tail:tt)*
    } => {
        py_error! { "__xor__ is not supported by py_class! yet." }
    };
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        def $name:ident (&$slf:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py, $name(&$slf,) $res_type; { $($body)* } [] }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_class_instance_method!{$py, $class::$name []};
        }; $($tail)*
    }};
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        def $name:ident (&$slf:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
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
        }; $($tail)*
    }};
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        @classmethod def $name:ident ($cls:ident) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info $slots
        /* impl: */ {
            $($imp)*
            py_class_impl_item! { $class, $py,$name($cls: &$crate::PyType,) $res_type; { $($body)* } [] }
        }
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = py_class_class_method!{$py, $class::$name []};
        }; $($tail)*
    }};
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        @classmethod def $name:ident ($cls:ident, $($p:tt)+) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
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
        }; $($tail)*
    }};
    { $class:ident $py:ident $info:tt $slots:tt
        { $( $imp:item )* }
        { $( $member_name:ident = $member_expr:expr; )* };
        @staticmethod def $name:ident ($($p:tt)*) -> $res_type:ty { $( $body:tt )* } $($tail:tt)*
    } => { py_class_impl! {
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
        }; $($tail)*
    }};
    { $class:ident $py:ident $info:tt $slots:tt $impls:tt
        { $( $member_name:ident = $member_expr:expr; )* };
        static $name:ident = $init:expr; $($tail:tt)*
    } => { py_class_impl! {
        $class $py $info $slots $impls
        /* members: */ {
            $( $member_name = $member_expr; )*
            $name = $init;
        }; $($tail)*
    }};

}

