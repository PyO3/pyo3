use ffi;

pub fn type_error_to_unit(py: ::Python, e: ::PyErr) -> ::PyResult<()> {
    if e.matches(py, py.get_type::<::exc::TypeError>()) {
        Ok(())
    } else {
        Err(e)
    }
}


#[macro_export]
#[doc(hidden)]
macro_rules! py_class_init_properties {
    ($class:ident, $py:ident, $type_object: ident, { }) => {{}};
    ($class:ident, $py:ident, $type_object: ident, { $( $prop:expr; )+ }) =>
    { unsafe {
        let mut defs = Vec::new();
        $(defs.push($prop);)+
        defs.push(
            $crate::_detail::ffi::PyGetSetDef {
                name: 0 as *mut $crate::_detail::libc::c_char,
                get: None,
                set: None,
                doc: 0 as *mut $crate::_detail::libc::c_char,
                closure: 0 as *mut $crate::_detail::libc::c_void,
            });
        let props = defs.into_boxed_slice();

        $type_object.tp_getset =
            props.as_ptr() as *mut $crate::_detail::ffi::PyGetSetDef;
        std::mem::forget(props);
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! py_class_property_impl {

    ({} $class:ident $py:ident $name:ident { $( $descr_name:ident = $descr_expr:expr; )* } ) =>
    {{
        let mut getset_def: $crate::_detail::ffi::PyGetSetDef =
            $crate::_detail::ffi::PyGetSetDef {
                name: 0 as *mut $crate::_detail::libc::c_char,
                get: None,
                set: None,
                doc: 0 as *mut $crate::_detail::libc::c_char,
                closure: 0 as *mut $crate::_detail::libc::c_void,
            };
        getset_def.name = concat!(stringify!($name), "\0").as_ptr() as *mut _;

        $( getset_def.$descr_name = Some($descr_expr); )*

        getset_def
    }};

    ( { get (&$slf:ident) -> $res_type:ty { $($body:tt)* } $($tail:tt)* }
        $class:ident $py:ident $name:ident { $( $descr_name:ident = $descr_expr:expr; )* } ) =>
    {
        py_class_property_impl!{
            { $($tail)* } $class $py $name
            /* methods: */ {
                $( $descr_name = $descr_expr; )*
                get = {
                    unsafe extern "C" fn wrap_getter_method(
                        slf: *mut $crate::_detail::ffi::PyObject,
                        _: *mut $crate::_detail::libc::c_void)
                        -> *mut $crate::_detail::ffi::PyObject
                    {
                        const LOCATION: &'static str = concat!(
                            stringify!($class), ".getter_", stringify!($name), "()");

                        fn get($slf: &$class, $py: $crate::Python) -> $res_type {
                            $($body)*
                        };

                        $crate::_detail::handle_callback(
                            LOCATION, $crate::_detail::PyObjectCallbackConverter,
                            |py| {
                                let slf = $crate::PyObject::from_borrowed_ptr(
                                    py, slf).unchecked_cast_into::<$class>();
                                let ret = get(&slf, py);
                                $crate::PyDrop::release_ref(slf, py);
                                ret
                            })
                    }
                    wrap_getter_method
                };
            }
        }
    };

    ( { set(&$slf:ident, $value:ident : $value_type:ty)
            -> $res_type:ty { $( $body:tt )* } $($tail:tt)* }
        $class:ident $py:ident $name:ident { $( $descr_name:ident = $descr_expr:expr; )* } ) =>
    {
        py_class_property_impl! {
            { $($tail)* } $class $py $name
            /* methods: */ {
                $( $descr_name = $descr_expr; )*
                set = {
                    unsafe extern "C" fn wrap_setter_method(
                        slf: *mut $crate::_detail::ffi::PyObject,
                        value: *mut $crate::_detail::ffi::PyObject,
                        _: *mut $crate::_detail::libc::c_void)
                        -> $crate::_detail::libc::c_int
                    {
                        const LOCATION: &'static str = concat!(
                            stringify!($class), ".setter_", stringify!($name), "()");

                        fn set($slf: &$class,
                               $py: $crate::Python, $value: $value_type) -> $res_type {
                            $($body)*
                        };

                        $crate::_detail::handle_callback(
                            LOCATION, $crate::py_class::slots::UnitCallbackConverter, move |py| {
                                let slf = $crate::PyObject::from_borrowed_ptr(py, slf)
                                    .unchecked_cast_into::<$class>();
                                let value = $crate::PyObject::from_borrowed_ptr(py, value);

                                let ret = match <$value_type as $crate::FromPyObject>::extract(py, &value) {
                                    Ok(value) => set(&slf, py, value),
                                    Err(e) =>
                                        $crate::py_class::properties::type_error_to_unit(py, e)
                                };
                                $crate::PyDrop::release_ref(slf, py);
                                $crate::PyDrop::release_ref(value, py);
                                ret
                            })
                    }
                    wrap_setter_method
                };
            }
        }
    };
}
