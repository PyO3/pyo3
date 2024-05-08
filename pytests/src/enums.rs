use pyo3::{
    pyclass, pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction_bound, Bound, PyResult,
};

#[pymodule]
pub fn enums(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SimpleEnum>()?;
    m.add_class::<ComplexEnum>()?;
    m.add_class::<TupleEnum>()?;
    m.add_class::<MixedComplexEnum>()?;
    m.add_wrapped(wrap_pyfunction_bound!(do_simple_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_complex_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_tuple_stuff))?;
    m.add_wrapped(wrap_pyfunction_bound!(do_mixed_complex_stuff))?;
    Ok(())
}

#[pyclass]
pub enum SimpleEnum {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[pyfunction]
pub fn do_simple_stuff(thing: &SimpleEnum) -> SimpleEnum {
    match thing {
        SimpleEnum::Sunday => SimpleEnum::Monday,
        SimpleEnum::Monday => SimpleEnum::Tuesday,
        SimpleEnum::Tuesday => SimpleEnum::Wednesday,
        SimpleEnum::Wednesday => SimpleEnum::Thursday,
        SimpleEnum::Thursday => SimpleEnum::Friday,
        SimpleEnum::Friday => SimpleEnum::Saturday,
        SimpleEnum::Saturday => SimpleEnum::Sunday,
    }
}

#[pyclass]
pub enum ComplexEnum {
    Int {
        i: i32,
    },
    Float {
        f: f64,
    },
    Str {
        s: String,
    },
    EmptyStruct {},
    MultiFieldStruct {
        a: i32,
        b: f64,
        c: bool,
    },
    #[pyo3(constructor = (a = 42, b = None))]
    VariantWithDefault {
        a: i32,
        b: Option<String>,
    },
}

#[pyfunction]
pub fn do_complex_stuff(thing: &ComplexEnum) -> ComplexEnum {
    match thing {
        ComplexEnum::Int { i } => ComplexEnum::Str { s: i.to_string() },
        ComplexEnum::Float { f } => ComplexEnum::Float { f: f * f },
        ComplexEnum::Str { s } => ComplexEnum::Int { i: s.len() as i32 },
        ComplexEnum::EmptyStruct {} => ComplexEnum::EmptyStruct {},
        ComplexEnum::MultiFieldStruct { a, b, c } => ComplexEnum::MultiFieldStruct {
            a: *a,
            b: *b,
            c: *c,
        },
        ComplexEnum::VariantWithDefault { a, b } => ComplexEnum::VariantWithDefault {
            a: 2 * a,
            b: b.as_ref().map(|s| s.to_uppercase()),
        },
    }
}

pub enum TupleEnum {
    Full(i32, f64, bool),
    EmptyTuple(),
}
#[allow(deprecated)]
unsafe impl ::pyo3::type_object::HasPyGilRef for TupleEnum {
    type AsRefTarget = ::pyo3::PyCell<Self>;
}
unsafe impl ::pyo3::type_object::PyTypeInfo for TupleEnum {
    const NAME: &'static str = "TupleEnum";
    const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
    #[inline]
    fn type_object_raw(py: ::pyo3::Python<'_>) -> *mut ::pyo3::ffi::PyTypeObject {
        use ::pyo3::prelude::PyTypeMethods;
        <TupleEnum as ::pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
            .get_or_init(py)
            .as_type_ptr()
    }
}
impl ::pyo3::PyClass for TupleEnum {
    type Frozen = ::pyo3::pyclass::boolean_struct::True;
}
impl<'a, 'py> ::pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a TupleEnum {
    type Holder = ::std::option::Option<::pyo3::PyRef<'py, TupleEnum>>;
    #[inline]
    fn extract(
        obj: &'a ::pyo3::Bound<'py, ::pyo3::PyAny>,
        holder: &'a mut Self::Holder,
    ) -> ::pyo3::PyResult<Self> {
        ::pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
    }
}
impl ::pyo3::IntoPy<::pyo3::PyObject> for TupleEnum {
    fn into_py(self, py: ::pyo3::Python) -> ::pyo3::PyObject {
        match self {
            TupleEnum::Full { .. } => {
                let pyclass_init =
                    ::pyo3::PyClassInitializer::from(self).add_subclass(TupleEnum_Full);
                let variant_value = ::pyo3::Py::new(py, pyclass_init).unwrap();
                ::pyo3::IntoPy::into_py(variant_value, py)
            }
            TupleEnum::EmptyTuple { .. } => {
                let pyclass_init =
                    ::pyo3::PyClassInitializer::from(self).add_subclass(TupleEnum_EmptyTuple);
                let variant_value = ::pyo3::Py::new(py, pyclass_init).unwrap();
                ::pyo3::IntoPy::into_py(variant_value, py)
            }
        }
    }
}
impl ::pyo3::impl_::pyclass::PyClassImpl for TupleEnum {
    const IS_BASETYPE: bool = true;
    const IS_SUBCLASS: bool = false;
    const IS_MAPPING: bool = false;
    const IS_SEQUENCE: bool = false;
    type BaseType = ::pyo3::PyAny;
    type ThreadChecker = ::pyo3::impl_::pyclass::SendablePyClass<TupleEnum>;
    type PyClassMutability = << :: pyo3 ::
               PyAny as :: pyo3 :: impl_ :: pyclass :: PyClassBaseType > ::
               PyClassMutability as :: pyo3 :: impl_ :: pycell :: PyClassMutability > ::
               ImmutableChild;
    type Dict = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type WeakRef = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type BaseNativeType = ::pyo3::PyAny;
    fn items_iter() -> ::pyo3::impl_::pyclass::PyClassItemsIter {
        use ::pyo3::impl_::pyclass::*;
        let collector = PyClassImplCollector::<Self>::new();
        static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
            methods: &[
                ::pyo3::class::PyMethodDefType::ClassAttribute({
                    ::pyo3::class::PyClassAttributeDef::new(
                        { "Full\0" },
                        ::pyo3::impl_::pymethods::PyClassAttributeFactory(
                            TupleEnum::__pymethod_variant_cls_Full__,
                        ),
                    )
                }),
                ::pyo3::class::PyMethodDefType::ClassAttribute({
                    ::pyo3::class::PyClassAttributeDef::new(
                        { "EmptyTuple\0" },
                        ::pyo3::impl_::pymethods::PyClassAttributeFactory(
                            TupleEnum::__pymethod_variant_cls_EmptyTuple__,
                        ),
                    )
                }),
            ],
            slots: &[],
        };
        PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
    }
    fn doc(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<&'static ::std::ffi::CStr> {
        use ::pyo3::impl_::pyclass::*;
        static DOC: ::pyo3::sync::GILOnceCell<::std::borrow::Cow<'static, ::std::ffi::CStr>> =
            ::pyo3::sync::GILOnceCell::new();
        DOC.get_or_try_init(py, || {
            let collector = PyClassImplCollector::<Self>::new();
            build_pyclass_doc(
                <TupleEnum as ::pyo3::PyTypeInfo>::NAME,
                "\0",
                collector.new_text_signature(),
            )
        })
        .map(::std::ops::Deref::deref)
    }
    fn lazy_type_object() -> &'static ::pyo3::impl_::pyclass::LazyTypeObject<Self> {
        use ::pyo3::impl_::pyclass::LazyTypeObject;
        static TYPE_OBJECT: LazyTypeObject<TupleEnum> = LazyTypeObject::new();
        &TYPE_OBJECT
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum {
    fn __pymethod_variant_cls_Full__(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<::pyo3::PyObject> {
        ::std::result::Result::Ok(py.get_type_bound::<TupleEnum_Full>().into_any().unbind())
    }
    fn __pymethod_variant_cls_EmptyTuple__(
        py: ::pyo3::Python<'_>,
    ) -> ::pyo3::PyResult<::pyo3::PyObject> {
        ::std::result::Result::Ok(
            py.get_type_bound::<TupleEnum_EmptyTuple>()
                .into_any()
                .unbind(),
        )
    }
}
impl TupleEnum {
    #[doc(hidden)]
    pub const _PYO3_DEF: ::pyo3::impl_::pymodule::AddClassToModule<Self> =
        ::pyo3::impl_::pymodule::AddClassToModule::new();
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum {}
#[doc(hidden)]
#[allow(non_camel_case_types)]
struct TupleEnum_Full;
#[doc(hidden)]
#[allow(non_camel_case_types)]
struct TupleEnum_EmptyTuple;
#[allow(deprecated)]
unsafe impl ::pyo3::type_object::HasPyGilRef for TupleEnum_Full {
    type AsRefTarget = ::pyo3::PyCell<Self>;
}
unsafe impl ::pyo3::type_object::PyTypeInfo for TupleEnum_Full {
    const NAME: &'static str = "TupleEnum_Full";
    const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
    #[inline]
    fn type_object_raw(py: ::pyo3::Python<'_>) -> *mut ::pyo3::ffi::PyTypeObject {
        use ::pyo3::prelude::PyTypeMethods;
        <TupleEnum_Full as ::pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
            .get_or_init(py)
            .as_type_ptr()
    }
}
#[allow(deprecated)]
unsafe impl ::pyo3::type_object::HasPyGilRef for TupleEnum_EmptyTuple {
    type AsRefTarget = ::pyo3::PyCell<Self>;
}
unsafe impl ::pyo3::type_object::PyTypeInfo for TupleEnum_EmptyTuple {
    const NAME: &'static str = "TupleEnum_EmptyTuple";
    const MODULE: ::std::option::Option<&'static str> = ::core::option::Option::None;
    #[inline]
    fn type_object_raw(py: ::pyo3::Python<'_>) -> *mut ::pyo3::ffi::PyTypeObject {
        use ::pyo3::prelude::PyTypeMethods;
        <TupleEnum_EmptyTuple as ::pyo3::impl_::pyclass::PyClassImpl>::lazy_type_object()
            .get_or_init(py)
            .as_type_ptr()
    }
}
impl ::pyo3::PyClass for TupleEnum_Full {
    type Frozen = ::pyo3::pyclass::boolean_struct::True;
}
impl<'a, 'py> ::pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py> for &'a TupleEnum_Full {
    type Holder = ::std::option::Option<::pyo3::PyRef<'py, TupleEnum_Full>>;
    #[inline]
    fn extract(
        obj: &'a ::pyo3::Bound<'py, ::pyo3::PyAny>,
        holder: &'a mut Self::Holder,
    ) -> ::pyo3::PyResult<Self> {
        ::pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
    }
}
impl ::pyo3::impl_::pyclass::PyClassImpl for TupleEnum_Full {
    const IS_BASETYPE: bool = false;
    const IS_SUBCLASS: bool = true;
    const IS_MAPPING: bool = false;
    const IS_SEQUENCE: bool = false;
    type BaseType = TupleEnum;
    type ThreadChecker = ::pyo3::impl_::pyclass::SendablePyClass<TupleEnum_Full>;
    type PyClassMutability = << TupleEnum
               as :: pyo3 :: impl_ :: pyclass :: PyClassBaseType > :: PyClassMutability
               as :: pyo3 :: impl_ :: pycell :: PyClassMutability > :: ImmutableChild;
    type Dict = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type WeakRef = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type BaseNativeType =
        <Self::BaseType as ::pyo3::impl_::pyclass::PyClassBaseType>::BaseNativeType;
    fn items_iter() -> ::pyo3::impl_::pyclass::PyClassItemsIter {
        use ::pyo3::impl_::pyclass::*;
        let collector = PyClassImplCollector::<Self>::new();
        static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
            methods: &[
                ::pyo3::class::PyMethodDefType::Getter(::pyo3::class::PyGetterDef::new(
                    "_0\0",
                    ::pyo3::impl_::pymethods::PyGetter(TupleEnum_Full::__pymethod_get__0__),
                    "\0",
                )),
                ::pyo3::class::PyMethodDefType::Getter(::pyo3::class::PyGetterDef::new(
                    "_1\0",
                    ::pyo3::impl_::pymethods::PyGetter(TupleEnum_Full::__pymethod_get__1__),
                    "\0",
                )),
                ::pyo3::class::PyMethodDefType::Getter(::pyo3::class::PyGetterDef::new(
                    "_2\0",
                    ::pyo3::impl_::pymethods::PyGetter(TupleEnum_Full::__pymethod_get__2__),
                    "\0",
                )),
                ::pyo3::class::PyMethodDefType::ClassAttribute({
                    ::pyo3::class::PyClassAttributeDef::new(
                        "__match_args__\0",
                        ::pyo3::impl_::pymethods::PyClassAttributeFactory(
                            TupleEnum_Full::__pymethod___match_args____,
                        ),
                    )
                }),
            ],
            slots: &[
                {
                    unsafe extern "C" fn trampoline(
                        _slf: *mut ::pyo3::ffi::PyObject,
                    ) -> ::pyo3::ffi::Py_ssize_t {
                        ::pyo3::impl_::trampoline::lenfunc(
                            _slf,
                            TupleEnum_Full::__pymethod___len____,
                        )
                    }
                    ::pyo3::ffi::PyType_Slot {
                        slot: ::pyo3::ffi::Py_mp_length,
                        pfunc: trampoline as ::pyo3::ffi::lenfunc as _,
                    }
                },
                {
                    unsafe extern "C" fn trampoline(
                        _slf: *mut ::pyo3::ffi::PyObject,
                        arg0: *mut ::pyo3::ffi::PyObject,
                    ) -> *mut ::pyo3::ffi::PyObject {
                        ::pyo3::impl_::trampoline::binaryfunc(
                            _slf,
                            arg0,
                            TupleEnum_Full::__pymethod___getitem____,
                        )
                    }
                    ::pyo3::ffi::PyType_Slot {
                        slot: ::pyo3::ffi::Py_mp_subscript,
                        pfunc: trampoline as ::pyo3::ffi::binaryfunc as _,
                    }
                },
                ::pyo3::ffi::PyType_Slot {
                    slot: ::pyo3::ffi::Py_tp_new,
                    pfunc: {
                        unsafe extern "C" fn trampoline(
                            subtype: *mut ::pyo3::ffi::PyTypeObject,
                            args: *mut ::pyo3::ffi::PyObject,
                            kwargs: *mut ::pyo3::ffi::PyObject,
                        ) -> *mut ::pyo3::ffi::PyObject {
                            use ::pyo3::impl_::pyclass::*;
                            #[allow(unknown_lints, non_local_definitions)]
                            impl PyClassNewTextSignature<TupleEnum_Full> for PyClassImplCollector<TupleEnum_Full> {
                                #[inline]
                                fn new_text_signature(self) -> ::std::option::Option<&'static str> {
                                    ::std::option::Option::Some("(_0, _1, _2)")
                                }
                            }
                            ::pyo3::impl_::trampoline::newfunc(
                                subtype,
                                args,
                                kwargs,
                                TupleEnum_Full::__pymethod___new____,
                            )
                        }
                        trampoline
                    } as ::pyo3::ffi::newfunc as _,
                },
            ],
        };
        PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
    }
    fn doc(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<&'static ::std::ffi::CStr> {
        use ::pyo3::impl_::pyclass::*;
        static DOC: ::pyo3::sync::GILOnceCell<::std::borrow::Cow<'static, ::std::ffi::CStr>> =
            ::pyo3::sync::GILOnceCell::new();
        DOC.get_or_try_init(py, || {
            let collector = PyClassImplCollector::<Self>::new();
            build_pyclass_doc(
                <TupleEnum_Full as ::pyo3::PyTypeInfo>::NAME,
                "\0",
                collector.new_text_signature(),
            )
        })
        .map(::std::ops::Deref::deref)
    }
    fn lazy_type_object() -> &'static ::pyo3::impl_::pyclass::LazyTypeObject<Self> {
        use ::pyo3::impl_::pyclass::LazyTypeObject;
        static TYPE_OBJECT: LazyTypeObject<TupleEnum_Full> = LazyTypeObject::new();
        &TYPE_OBJECT
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum_Full {
    unsafe fn __pymethod_get__0__(
        py: ::pyo3::Python<'_>,
        _slf: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        #[allow(clippy::let_unit_value)]
        let result = ::pyo3::callback::convert(
            py,
            TupleEnum_Full::_0(
                ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                    .downcast::<TupleEnum_Full>()
                    .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                    .and_then(
                        #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                        |bound| {
                            ::std::convert::TryFrom::try_from(bound)
                                .map_err(::std::convert::Into::into)
                        },
                    )?,
            ),
        );
        result
    }
    unsafe fn __pymethod_get__1__(
        py: ::pyo3::Python<'_>,
        _slf: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        #[allow(clippy::let_unit_value)]
        let result = ::pyo3::callback::convert(
            py,
            TupleEnum_Full::_1(
                ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                    .downcast::<TupleEnum_Full>()
                    .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                    .and_then(
                        #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                        |bound| {
                            ::std::convert::TryFrom::try_from(bound)
                                .map_err(::std::convert::Into::into)
                        },
                    )?,
            ),
        );
        result
    }
    unsafe fn __pymethod_get__2__(
        py: ::pyo3::Python<'_>,
        _slf: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        #[allow(clippy::let_unit_value)]
        let result = ::pyo3::callback::convert(
            py,
            TupleEnum_Full::_2(
                ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                    .downcast::<TupleEnum_Full>()
                    .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                    .and_then(
                        #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                        |bound| {
                            ::std::convert::TryFrom::try_from(bound)
                                .map_err(::std::convert::Into::into)
                        },
                    )?,
            ),
        );
        result
    }
    fn __pymethod___match_args____(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<::pyo3::PyObject> {
        let function = TupleEnum_Full::__match_args__;
        ::pyo3::impl_::wrap::map_result_into_py(
            py,
            ::pyo3::impl_::wrap::OkWrap::wrap(function())
                .map_err(::core::convert::Into::<::pyo3::PyErr>::into),
        )
    }
    unsafe fn __pymethod___len____(
        py: ::pyo3::Python<'_>,
        _raw_slf: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<::pyo3::ffi::Py_ssize_t> {
        let function = TupleEnum_Full::__len__;
        let _slf = _raw_slf;
        #[allow(clippy::let_unit_value)]
        let result = TupleEnum_Full::__len__(
            ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                .downcast::<TupleEnum_Full>()
                .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                .and_then(
                    #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                    |bound| {
                        ::std::convert::TryFrom::try_from(bound).map_err(::std::convert::Into::into)
                    },
                )?,
        );
        ::pyo3::callback::convert(py, result)
    }
    unsafe fn __pymethod___getitem____(
        py: ::pyo3::Python<'_>,
        _raw_slf: *mut ::pyo3::ffi::PyObject,
        arg0: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        let function = TupleEnum_Full::__getitem__;
        let _slf = _raw_slf;
        #[allow(clippy::let_unit_value)]
        let mut holder_0 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        let gil_refs_checker_0 = ::pyo3::impl_::deprecations::GilRefs::new();
        let result = TupleEnum_Full::__getitem__(
            ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                .downcast::<TupleEnum_Full>()
                .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                .and_then(
                    #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                    |bound| {
                        ::std::convert::TryFrom::try_from(bound).map_err(::std::convert::Into::into)
                    },
                )?,
            ::pyo3::impl_::deprecations::inspect_type(
                ::pyo3::impl_::extract_argument::extract_argument(
                    ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &arg0).0,
                    &mut holder_0,
                    "idx",
                )?,
                &gil_refs_checker_0,
            ),
        );
        gil_refs_checker_0.function_arg();
        ::pyo3::callback::convert(py, result)
    }
    unsafe fn __pymethod___new____(
        py: ::pyo3::Python<'_>,
        _slf: *mut ::pyo3::ffi::PyTypeObject,
        _args: *mut ::pyo3::ffi::PyObject,
        _kwargs: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        use ::pyo3::callback::IntoPyCallbackOutput;
        let _slf_ref = &_slf;
        let function = TupleEnum_Full::__pymethod_constructor__;
        const DESCRIPTION: ::pyo3::impl_::extract_argument::FunctionDescription =
            ::pyo3::impl_::extract_argument::FunctionDescription {
                cls_name: ::std::option::Option::Some(
                    <TupleEnum_Full as ::pyo3::type_object::PyTypeInfo>::NAME,
                ),
                func_name: stringify!(__new__),
                positional_parameter_names: &["_0", "_1", "_2"],
                positional_only_parameters: 0usize,
                required_positional_parameters: 3usize,
                keyword_only_parameters: &[],
            };
        let mut output = [::std::option::Option::None; 3usize];
        let
                   (_args, _kwargs) = DESCRIPTION.extract_arguments_tuple_dict :: < ::
                   pyo3 :: impl_ :: extract_argument :: NoVarargs, :: pyo3 :: impl_ ::
                   extract_argument :: NoVarkeywords > (py, _args, _kwargs, & mut output)
                   ? ;
        #[allow(clippy::let_unit_value)]
        let mut holder_0 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        let mut holder_1 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        let mut holder_2 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        let gil_refs_checker_0 = ::pyo3::impl_::deprecations::GilRefs::new();
        let gil_refs_checker_1 = ::pyo3::impl_::deprecations::GilRefs::new();
        let gil_refs_checker_2 = ::pyo3::impl_::deprecations::GilRefs::new();
        let result = TupleEnum_Full::__pymethod_constructor__(
            py,
            ::pyo3::impl_::deprecations::inspect_type(
                ::pyo3::impl_::extract_argument::extract_argument(
                    ::pyo3::impl_::extract_argument::unwrap_required_argument(
                        output[0usize].as_deref(),
                    ),
                    &mut holder_0,
                    "_0",
                )?,
                &gil_refs_checker_0,
            ),
            ::pyo3::impl_::deprecations::inspect_type(
                ::pyo3::impl_::extract_argument::extract_argument(
                    ::pyo3::impl_::extract_argument::unwrap_required_argument(
                        output[1usize].as_deref(),
                    ),
                    &mut holder_1,
                    "_1",
                )?,
                &gil_refs_checker_1,
            ),
            ::pyo3::impl_::deprecations::inspect_type(
                ::pyo3::impl_::extract_argument::extract_argument(
                    ::pyo3::impl_::extract_argument::unwrap_required_argument(
                        output[2usize].as_deref(),
                    ),
                    &mut holder_2,
                    "_2",
                )?,
                &gil_refs_checker_2,
            ),
        );
        let initializer: ::pyo3::PyClassInitializer<TupleEnum_Full> = result.convert(py)?;
        gil_refs_checker_0.function_arg();
        gil_refs_checker_1.function_arg();
        gil_refs_checker_2.function_arg();
        ::pyo3::impl_::pymethods::tp_new_impl(py, initializer, _slf)
    }
}
impl TupleEnum_Full {
    #[doc(hidden)]
    pub const _PYO3_DEF: ::pyo3::impl_::pymodule::AddClassToModule<Self> =
        ::pyo3::impl_::pymodule::AddClassToModule::new();
}
impl ::pyo3::PyClass for TupleEnum_EmptyTuple {
    type Frozen = ::pyo3::pyclass::boolean_struct::True;
}
impl<'a, 'py> ::pyo3::impl_::extract_argument::PyFunctionArgument<'a, 'py>
    for &'a TupleEnum_EmptyTuple
{
    type Holder = ::std::option::Option<::pyo3::PyRef<'py, TupleEnum_EmptyTuple>>;
    #[inline]
    fn extract(
        obj: &'a ::pyo3::Bound<'py, ::pyo3::PyAny>,
        holder: &'a mut Self::Holder,
    ) -> ::pyo3::PyResult<Self> {
        ::pyo3::impl_::extract_argument::extract_pyclass_ref(obj, holder)
    }
}
impl ::pyo3::impl_::pyclass::PyClassImpl for TupleEnum_EmptyTuple {
    const IS_BASETYPE: bool = false;
    const IS_SUBCLASS: bool = true;
    const IS_MAPPING: bool = false;
    const IS_SEQUENCE: bool = false;
    type BaseType = TupleEnum;
    type ThreadChecker = ::pyo3::impl_::pyclass::SendablePyClass<TupleEnum_EmptyTuple>;
    type PyClassMutability = <<
               TupleEnum as :: pyo3 :: impl_ :: pyclass :: PyClassBaseType > ::
               PyClassMutability as :: pyo3 :: impl_ :: pycell :: PyClassMutability > ::
               ImmutableChild;
    type Dict = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type WeakRef = ::pyo3::impl_::pyclass::PyClassDummySlot;
    type BaseNativeType =
        <Self::BaseType as ::pyo3::impl_::pyclass::PyClassBaseType>::BaseNativeType;
    fn items_iter() -> ::pyo3::impl_::pyclass::PyClassItemsIter {
        use ::pyo3::impl_::pyclass::*;
        let collector = PyClassImplCollector::<Self>::new();
        static INTRINSIC_ITEMS: PyClassItems = PyClassItems {
            methods: &[::pyo3::class::PyMethodDefType::ClassAttribute({
                ::pyo3::class::PyClassAttributeDef::new(
                    "__match_args__\0",
                    ::pyo3::impl_::pymethods::PyClassAttributeFactory(
                        TupleEnum_EmptyTuple::__pymethod___match_args____,
                    ),
                )
            })],
            slots: &[
                {
                    unsafe extern "C" fn trampoline(
                        _slf: *mut ::pyo3::ffi::PyObject,
                    ) -> ::pyo3::ffi::Py_ssize_t {
                        ::pyo3::impl_::trampoline::lenfunc(
                            _slf,
                            TupleEnum_EmptyTuple::__pymethod___len____,
                        )
                    }
                    ::pyo3::ffi::PyType_Slot {
                        slot: ::pyo3::ffi::Py_mp_length,
                        pfunc: trampoline as ::pyo3::ffi::lenfunc as _,
                    }
                },
                {
                    unsafe extern "C" fn trampoline(
                        _slf: *mut ::pyo3::ffi::PyObject,
                        arg0: *mut ::pyo3::ffi::PyObject,
                    ) -> *mut ::pyo3::ffi::PyObject {
                        ::pyo3::impl_::trampoline::binaryfunc(
                            _slf,
                            arg0,
                            TupleEnum_EmptyTuple::__pymethod___getitem____,
                        )
                    }
                    ::pyo3::ffi::PyType_Slot {
                        slot: ::pyo3::ffi::Py_mp_subscript,
                        pfunc: trampoline as ::pyo3::ffi::binaryfunc as _,
                    }
                },
                ::pyo3::ffi::PyType_Slot {
                    slot: ::pyo3::ffi::Py_tp_new,
                    pfunc: {
                        unsafe extern "C" fn trampoline(
                            subtype: *mut ::pyo3::ffi::PyTypeObject,
                            args: *mut ::pyo3::ffi::PyObject,
                            kwargs: *mut ::pyo3::ffi::PyObject,
                        ) -> *mut ::pyo3::ffi::PyObject {
                            use ::pyo3::impl_::pyclass::*;
                            #[allow(unknown_lints, non_local_definitions)]
                            impl PyClassNewTextSignature<TupleEnum_EmptyTuple> for PyClassImplCollector<TupleEnum_EmptyTuple> {
                                #[inline]
                                fn new_text_signature(self) -> ::std::option::Option<&'static str> {
                                    ::std::option::Option::Some("()")
                                }
                            }
                            ::pyo3::impl_::trampoline::newfunc(
                                subtype,
                                args,
                                kwargs,
                                TupleEnum_EmptyTuple::__pymethod___new____,
                            )
                        }
                        trampoline
                    } as ::pyo3::ffi::newfunc as _,
                },
            ],
        };
        PyClassItemsIter::new(&INTRINSIC_ITEMS, collector.py_methods())
    }
    fn doc(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<&'static ::std::ffi::CStr> {
        use ::pyo3::impl_::pyclass::*;
        static DOC: ::pyo3::sync::GILOnceCell<::std::borrow::Cow<'static, ::std::ffi::CStr>> =
            ::pyo3::sync::GILOnceCell::new();
        DOC.get_or_try_init(py, || {
            let collector = PyClassImplCollector::<Self>::new();
            build_pyclass_doc(
                <TupleEnum_EmptyTuple as ::pyo3::PyTypeInfo>::NAME,
                "\0",
                collector.new_text_signature(),
            )
        })
        .map(::std::ops::Deref::deref)
    }
    fn lazy_type_object() -> &'static ::pyo3::impl_::pyclass::LazyTypeObject<Self> {
        use ::pyo3::impl_::pyclass::LazyTypeObject;
        static TYPE_OBJECT: LazyTypeObject<TupleEnum_EmptyTuple> = LazyTypeObject::new();
        &TYPE_OBJECT
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum_EmptyTuple {
    fn __pymethod___match_args____(py: ::pyo3::Python<'_>) -> ::pyo3::PyResult<::pyo3::PyObject> {
        let function = TupleEnum_EmptyTuple::__match_args__;
        ::pyo3::impl_::wrap::map_result_into_py(
            py,
            ::pyo3::impl_::wrap::OkWrap::wrap(function())
                .map_err(::core::convert::Into::<::pyo3::PyErr>::into),
        )
    }
    unsafe fn __pymethod___len____(
        py: ::pyo3::Python<'_>,
        _raw_slf: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<::pyo3::ffi::Py_ssize_t> {
        let function = TupleEnum_EmptyTuple::__len__;
        let _slf = _raw_slf;
        #[allow(clippy::let_unit_value)]
        let result = TupleEnum_EmptyTuple::__len__(
            ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                .downcast::<TupleEnum_EmptyTuple>()
                .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                .and_then(
                    #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                    |bound| {
                        ::std::convert::TryFrom::try_from(bound).map_err(::std::convert::Into::into)
                    },
                )?,
        );
        ::pyo3::callback::convert(py, result)
    }
    unsafe fn __pymethod___getitem____(
        py: ::pyo3::Python<'_>,
        _raw_slf: *mut ::pyo3::ffi::PyObject,
        arg0: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        let function = TupleEnum_EmptyTuple::__getitem__;
        let _slf = _raw_slf;
        #[allow(clippy::let_unit_value)]
        let mut holder_0 = ::pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        let gil_refs_checker_0 = ::pyo3::impl_::deprecations::GilRefs::new();
        let result = TupleEnum_EmptyTuple::__getitem__(
            ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &_slf)
                .downcast::<TupleEnum_EmptyTuple>()
                .map_err(::std::convert::Into::<::pyo3::PyErr>::into)
                .and_then(
                    #[allow(unknown_lints, clippy::unnecessary_fallible_conversions)]
                    |bound| {
                        ::std::convert::TryFrom::try_from(bound).map_err(::std::convert::Into::into)
                    },
                )?,
            ::pyo3::impl_::deprecations::inspect_type(
                ::pyo3::impl_::extract_argument::extract_argument(
                    ::pyo3::impl_::pymethods::BoundRef::ref_from_ptr(py, &arg0).0,
                    &mut holder_0,
                    "idx",
                )?,
                &gil_refs_checker_0,
            ),
        );
        gil_refs_checker_0.function_arg();
        ::pyo3::callback::convert(py, result)
    }
    unsafe fn __pymethod___new____(
        py: ::pyo3::Python<'_>,
        _slf: *mut ::pyo3::ffi::PyTypeObject,
        _args: *mut ::pyo3::ffi::PyObject,
        _kwargs: *mut ::pyo3::ffi::PyObject,
    ) -> ::pyo3::PyResult<*mut ::pyo3::ffi::PyObject> {
        use ::pyo3::callback::IntoPyCallbackOutput;
        let _slf_ref = &_slf;
        let function = TupleEnum_EmptyTuple::__pymethod_constructor__;
        const DESCRIPTION: ::pyo3::impl_::extract_argument::FunctionDescription =
            ::pyo3::impl_::extract_argument::FunctionDescription {
                cls_name: ::std::option::Option::Some(
                    <TupleEnum_EmptyTuple as ::pyo3::type_object::PyTypeInfo>::NAME,
                ),
                func_name: stringify!(__new__),
                positional_parameter_names: &[],
                positional_only_parameters: 0usize,
                required_positional_parameters: 0usize,
                keyword_only_parameters: &[],
            };
        let mut output = [::std::option::Option::None; 0usize];
        let
                   (_args, _kwargs) = DESCRIPTION.extract_arguments_tuple_dict :: < ::
                   pyo3 :: impl_ :: extract_argument :: NoVarargs, :: pyo3 :: impl_ ::
                   extract_argument :: NoVarkeywords > (py, _args, _kwargs, & mut output)
                   ? ;
        #[allow(clippy::let_unit_value)]
        let result = TupleEnum_EmptyTuple::__pymethod_constructor__(py);
        let initializer: ::pyo3::PyClassInitializer<TupleEnum_EmptyTuple> = result.convert(py)?;
        ::pyo3::impl_::pymethods::tp_new_impl(py, initializer, _slf)
    }
}
impl TupleEnum_EmptyTuple {
    #[doc(hidden)]
    pub const _PYO3_DEF: ::pyo3::impl_::pymodule::AddClassToModule<Self> =
        ::pyo3::impl_::pymodule::AddClassToModule::new();
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum_Full {
    fn __pymethod_constructor__(
        py: ::pyo3::Python<'_>,
        _0: i32,
        _1: f64,
        _2: bool,
    ) -> ::pyo3::PyClassInitializer<TupleEnum_Full> {
        let base_value = TupleEnum::Full(_0, _1, _2);
        ::pyo3::PyClassInitializer::from(base_value).add_subclass(TupleEnum_Full)
    }
    fn __len__(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<usize> {
        Ok(3usize)
    }
    fn __getitem__(slf: ::pyo3::PyRef<Self>, idx: usize) -> ::pyo3::PyResult<::pyo3::PyObject> {
        let py = slf.py();
        match idx {
            0usize => Ok(::pyo3::IntoPy::into_py(TupleEnum_Full::_0(slf)?, py)),
            1usize => Ok(::pyo3::IntoPy::into_py(TupleEnum_Full::_1(slf)?, py)),
            2usize => Ok(::pyo3::IntoPy::into_py(TupleEnum_Full::_2(slf)?, py)),
            _ => Err(pyo3::exceptions::PyIndexError::new_err(
                "tuple index out of range",
            )),
        }
    }
    fn __match_args__(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<(i32, f64, bool)> {
        match &*slf.into_super() {
            TupleEnum::Full(_0, _1, _2) => Ok((_0.clone(), _1.clone(), _2.clone())),
            _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
        }
    }
    fn _0(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<i32> {
        match &*slf.into_super() {
            TupleEnum::Full(val, _, _) => Ok(val.clone()),
            _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
        }
    }
    fn _1(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<f64> {
        match &*slf.into_super() {
            TupleEnum::Full(_, val, _) => Ok(val.clone()),
            _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
        }
    }
    fn _2(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<bool> {
        match &*slf.into_super() {
            TupleEnum::Full(_, _, val) => Ok(val.clone()),
            _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
        }
    }
}
#[doc(hidden)]
#[allow(non_snake_case)]
impl TupleEnum_EmptyTuple {
    fn __pymethod_constructor__(
        py: ::pyo3::Python<'_>,
    ) -> ::pyo3::PyClassInitializer<TupleEnum_EmptyTuple> {
        let base_value = TupleEnum::EmptyTuple();
        ::pyo3::PyClassInitializer::from(base_value).add_subclass(TupleEnum_EmptyTuple)
    }
    fn __len__(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<usize> {
        Ok(0usize)
    }
    fn __getitem__(slf: ::pyo3::PyRef<Self>, idx: usize) -> ::pyo3::PyResult<::pyo3::PyObject> {
        let py = slf.py();
        match idx {
            _ => Err(pyo3::exceptions::PyIndexError::new_err(
                "tuple index out of range",
            )),
        }
    }
    fn __match_args__(slf: ::pyo3::PyRef<Self>) -> ::pyo3::PyResult<()> {
        match &*slf.into_super() {
            TupleEnum::EmptyTuple() => Ok(()),
            _ => unreachable!("Wrong complex enum variant found in variant wrapper PyClass"),
        }
    }
}

#[pyfunction]
pub fn do_tuple_stuff(thing: &TupleEnum) -> TupleEnum {
    match thing {
        TupleEnum::Full(a, b, c) => TupleEnum::Full(*a, *b, *c),
        TupleEnum::EmptyTuple() => TupleEnum::EmptyTuple(),
    }
}

#[pyclass]
pub enum MixedComplexEnum {
    Nothing {},
    Empty(),
}

#[pyfunction]
pub fn do_mixed_complex_stuff(thing: &MixedComplexEnum) -> MixedComplexEnum {
    match thing {
        MixedComplexEnum::Nothing {} => MixedComplexEnum::Empty(),
        MixedComplexEnum::Empty() => MixedComplexEnum::Nothing {},
    }
}
