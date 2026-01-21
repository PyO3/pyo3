use crate::{
    exceptions::{PyAttributeError, PyNotImplementedError, PyRuntimeError},
    ffi,
    ffi_ptr_ext::FfiPtrExt,
    impl_::{
        freelist::PyObjectFreeList,
        pycell::{GetBorrowChecker, PyClassMutability, PyClassObjectBaseLayout},
        pyclass_init::PyObjectInit,
        pymethods::{PyGetterDef, PyMethodDefType},
    },
    pycell::{impl_::PyClassObjectLayout, PyBorrowError},
    types::{any::PyAnyMethods, PyBool},
    Borrowed, IntoPyObject, IntoPyObjectExt, Py, PyAny, PyClass, PyClassGuard, PyErr, PyResult,
    PyTypeCheck, PyTypeInfo, Python,
};
use std::{
    ffi::CStr,
    marker::PhantomData,
    os::raw::{c_int, c_void},
    ptr::{self, NonNull},
    sync::Mutex,
    thread,
};

mod assertions;
pub mod doc;
mod lazy_type_object;
#[macro_use]
mod probes;

pub use assertions::*;
pub use lazy_type_object::{type_object_init_failed, LazyTypeObject};
pub use probes::*;

/// Gets the offset of the dictionary from the start of the object in bytes.
#[inline]
pub const fn dict_offset<T: PyClass>() -> PyObjectOffset {
    <T as PyClassImpl>::Layout::DICT_OFFSET
}

/// Gets the offset of the weakref list from the start of the object in bytes.
#[inline]
pub const fn weaklist_offset<T: PyClass>() -> PyObjectOffset {
    <T as PyClassImpl>::Layout::WEAKLIST_OFFSET
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::PyClassDummySlot {}
    impl Sealed for super::PyClassDictSlot {}
    impl Sealed for super::PyClassWeakRefSlot {}
    impl Sealed for super::ThreadCheckerImpl {}
    impl Sealed for super::NoopThreadChecker {}
}

/// Represents the `__dict__` field for `#[pyclass]`.
pub trait PyClassDict: sealed::Sealed {
    /// Initial form of a [PyObject](crate::ffi::PyObject) `__dict__` reference.
    const INIT: Self;
    /// Empties the dictionary of its key-value pairs.
    #[inline]
    fn clear_dict(&mut self, _py: Python<'_>) {}
}

/// Represents the `__weakref__` field for `#[pyclass]`.
pub trait PyClassWeakRef: sealed::Sealed {
    /// Initializes a `weakref` instance.
    const INIT: Self;
    /// Clears the weak references to the given object.
    ///
    /// # Safety
    /// - `_obj` must be a pointer to the pyclass instance which contains `self`.
    /// - The GIL must be held.
    #[inline]
    unsafe fn clear_weakrefs(&mut self, _obj: *mut ffi::PyObject, _py: Python<'_>) {}
}

/// Zero-sized dummy field.
pub struct PyClassDummySlot;

impl PyClassDict for PyClassDummySlot {
    const INIT: Self = PyClassDummySlot;
}

impl PyClassWeakRef for PyClassDummySlot {
    const INIT: Self = PyClassDummySlot;
}

/// Actual dict field, which holds the pointer to `__dict__`.
///
/// `#[pyclass(dict)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassDictSlot(*mut ffi::PyObject);

impl PyClassDict for PyClassDictSlot {
    const INIT: Self = Self(std::ptr::null_mut());
    #[inline]
    fn clear_dict(&mut self, _py: Python<'_>) {
        if !self.0.is_null() {
            unsafe { ffi::PyDict_Clear(self.0) }
        }
    }
}

/// Actual weakref field, which holds the pointer to `__weakref__`.
///
/// `#[pyclass(weakref)]` automatically adds this.
#[repr(transparent)]
pub struct PyClassWeakRefSlot(*mut ffi::PyObject);

impl PyClassWeakRef for PyClassWeakRefSlot {
    const INIT: Self = Self(std::ptr::null_mut());
    #[inline]
    unsafe fn clear_weakrefs(&mut self, obj: *mut ffi::PyObject, _py: Python<'_>) {
        if !self.0.is_null() {
            unsafe { ffi::PyObject_ClearWeakRefs(obj) }
        }
    }
}

/// This type is used as a "dummy" type on which dtolnay specializations are
/// applied to apply implementations from `#[pymethods]`
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
        *self
    }
}

impl<T> Copy for PyClassImplCollector<T> {}

pub struct PyClassItems {
    pub methods: &'static [PyMethodDefType],
    pub slots: &'static [ffi::PyType_Slot],
}

// Allow PyClassItems in statics
unsafe impl Sync for PyClassItems {}

/// Implements the underlying functionality of `#[pyclass]`, assembled by various proc macros.
///
/// Users are discouraged from implementing this trait manually; it is a PyO3 implementation detail
/// and may be changed at any time.
pub trait PyClassImpl: Sized + 'static {
    /// Module which the class will be associated with.
    ///
    /// (Currently defaults to `builtins` if unset, this will likely be improved in the future, it
    /// may also be removed when passing module objects in class init.)
    const MODULE: Option<&'static str>;

    /// #[pyclass(subclass)]
    const IS_BASETYPE: bool = false;

    /// #[pyclass(extends=...)]
    const IS_SUBCLASS: bool = false;

    /// #[pyclass(mapping)]
    const IS_MAPPING: bool = false;

    /// #[pyclass(sequence)]
    const IS_SEQUENCE: bool = false;

    /// #[pyclass(immutable_type)]
    const IS_IMMUTABLE_TYPE: bool = false;

    /// Description of how this class is laid out in memory
    type Layout: PyClassObjectLayout<Self>;

    /// Base class
    type BaseType: PyTypeInfo + PyClassBaseType;

    /// Immutable or mutable
    type PyClassMutability: PyClassMutability + GetBorrowChecker<Self>;

    /// Specify this class has `#[pyclass(dict)]` or not.
    type Dict: PyClassDict;

    /// Specify this class has `#[pyclass(weakref)]` or not.
    type WeakRef: PyClassWeakRef;

    /// The closest native ancestor. This is `PyAny` by default, and when you declare
    /// `#[pyclass(extends=PyDict)]`, it's `PyDict`.
    type BaseNativeType: PyTypeInfo;

    /// This handles following two situations:
    /// 1. In case `T` is `Send`, stub `ThreadChecker` is used and does nothing.
    ///    This implementation is used by default. Compile fails if `T: !Send`.
    /// 2. In case `T` is `!Send`, `ThreadChecker` panics when `T` is accessed by another thread.
    ///    This implementation is used when `#[pyclass(unsendable)]` is given.
    ///    Panicking makes it safe to expose `T: !Send` to the Python interpreter, where all objects
    ///    can be accessed by multiple threads by `threading` module.
    type ThreadChecker: PyClassThreadChecker<Self>;

    #[cfg(feature = "multiple-pymethods")]
    type Inventory: PyClassInventory;

    /// Docstring for the class provided on the struct or enum.
    ///
    /// This is exposed for `PyClassDocGenerator` to use as a docstring piece.
    const RAW_DOC: &'static CStr;

    /// Fully rendered class doc, including the `text_signature` if a constructor is defined.
    ///
    /// This is constructed at compile-time with const specialization via the proc macros with help
    /// from the PyClassDocGenerator` type.
    const DOC: &'static CStr;

    fn items_iter() -> PyClassItemsIter;

    /// Used to provide the __dictoffset__ slot
    /// (equivalent to [tp_dictoffset](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_dictoffset))
    #[inline]
    fn dict_offset() -> Option<PyObjectOffset> {
        None
    }

    /// Used to provide the __weaklistoffset__ slot
    /// (equivalent to [tp_weaklistoffset](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_weaklistoffset)
    #[inline]
    fn weaklist_offset() -> Option<PyObjectOffset> {
        None
    }

    fn lazy_type_object() -> &'static LazyTypeObject<Self>;
}

/// Iterator used to process all class items during type instantiation.
pub struct PyClassItemsIter {
    /// Iteration state
    idx: usize,
    /// Items from the `#[pyclass]` macro
    pyclass_items: &'static PyClassItems,
    /// Items from the `#[pymethods]` macro
    #[cfg(not(feature = "multiple-pymethods"))]
    pymethods_items: &'static PyClassItems,
    /// Items from the `#[pymethods]` macro with inventory
    #[cfg(feature = "multiple-pymethods")]
    pymethods_items: Box<dyn Iterator<Item = &'static PyClassItems>>,
}

impl PyClassItemsIter {
    pub fn new(
        pyclass_items: &'static PyClassItems,
        #[cfg(not(feature = "multiple-pymethods"))] pymethods_items: &'static PyClassItems,
        #[cfg(feature = "multiple-pymethods")] pymethods_items: Box<
            dyn Iterator<Item = &'static PyClassItems>,
        >,
    ) -> Self {
        Self {
            idx: 0,
            pyclass_items,
            pymethods_items,
        }
    }
}

impl Iterator for PyClassItemsIter {
    type Item = &'static PyClassItems;

    #[cfg(not(feature = "multiple-pymethods"))]
    fn next(&mut self) -> Option<Self::Item> {
        match self.idx {
            0 => {
                self.idx += 1;
                Some(self.pyclass_items)
            }
            1 => {
                self.idx += 1;
                Some(self.pymethods_items)
            }
            // Termination clause
            _ => None,
        }
    }

    #[cfg(feature = "multiple-pymethods")]
    fn next(&mut self) -> Option<Self::Item> {
        match self.idx {
            0 => {
                self.idx += 1;
                Some(self.pyclass_items)
            }
            // Termination clause
            _ => self.pymethods_items.next(),
        }
    }
}

// Traits describing known special methods.

macro_rules! slot_fragment_trait {
    ($trait_name:ident, $($default_method:tt)*) => {
        #[allow(non_camel_case_types, reason = "to match Python dunder names")]
        pub trait $trait_name<T>: Sized {
            $($default_method)*
        }

        impl<T> $trait_name<T> for &'_ PyClassImplCollector<T> {}
    }
}

slot_fragment_trait! {
    PyClass__getattribute__SlotFragment,

    /// # Safety: _slf and _attr must be valid non-null Python objects
    #[inline]
    unsafe fn __getattribute__(
        self,
        py: Python<'_>,
        slf: *mut ffi::PyObject,
        attr: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        let res = unsafe { ffi::PyObject_GenericGetAttr(slf, attr) };
        if res.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(res)
        }
    }
}

slot_fragment_trait! {
    PyClass__getattr__SlotFragment,

    /// # Safety: _slf and _attr must be valid non-null Python objects
    #[inline]
    unsafe fn __getattr__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        attr: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Err(PyErr::new::<PyAttributeError, _>(
            // SAFETY: caller has upheld the safety contract
            (unsafe { attr.assume_borrowed_unchecked(py) }.to_owned().unbind(),)
        ))
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_getattro_slot {
    ($cls:ty) => {{
        unsafe fn slot_impl(
            py: $crate::Python<'_>,
            _slf: *mut $crate::ffi::PyObject,
            attr: *mut $crate::ffi::PyObject,
        ) -> $crate::PyResult<*mut $crate::ffi::PyObject> {
            use ::std::result::Result::*;
            use $crate::impl_::pyclass::*;
            let collector = PyClassImplCollector::<$cls>::new();

            // Strategy:
            // - Try __getattribute__ first. Its default is PyObject_GenericGetAttr.
            // - If it returns a result, use it.
            // - If it fails with AttributeError, try __getattr__.
            // - If it fails otherwise, reraise.
            match unsafe { collector.__getattribute__(py, _slf, attr) } {
                Ok(obj) => Ok(obj),
                Err(e) if e.is_instance_of::<$crate::exceptions::PyAttributeError>(py) => unsafe {
                    collector.__getattr__(py, _slf, attr)
                },
                Err(e) => Err(e),
            }
        }

        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_tp_getattro,
            pfunc: $crate::impl_::trampoline::get_trampoline_function!(getattrofunc, slot_impl)
                as $crate::ffi::getattrofunc as _,
        }
    }};
}

pub use generate_pyclass_getattro_slot;

/// Macro which expands to three items
/// - Trait for a __setitem__ dunder
/// - Trait for the corresponding __delitem__ dunder
/// - A macro which will use dtolnay specialisation to generate the shared slot for the two dunders
macro_rules! define_pyclass_setattr_slot {
    (
        $set_trait:ident,
        $del_trait:ident,
        $set:ident,
        $del:ident,
        $set_error:expr,
        $del_error:expr,
        $generate_macro:ident,
        $slot:ident,
        $func_ty:ident,
    ) => {
        slot_fragment_trait! {
            $set_trait,

            /// # Safety: _slf and _attr must be valid non-null Python objects
            #[inline]
            unsafe fn $set(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _attr: *mut ffi::PyObject,
                _value: NonNull<ffi::PyObject>,
            ) -> PyResult<()> {
                $set_error
            }
        }

        slot_fragment_trait! {
            $del_trait,

            /// # Safety: _slf and _attr must be valid non-null Python objects
            #[inline]
            unsafe fn $del(
                self,
                _py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _attr: *mut ffi::PyObject,
            ) -> PyResult<()> {
                $del_error
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! $generate_macro {
            ($cls:ty) => {{
                unsafe fn slot_impl(
                    py: $crate::Python<'_>,
                    _slf: *mut $crate::ffi::PyObject,
                    attr: *mut $crate::ffi::PyObject,
                    value: *mut $crate::ffi::PyObject,
                ) -> $crate::PyResult<::std::ffi::c_int> {
                    use ::std::option::Option::*;
                    use $crate::impl_::callback::IntoPyCallbackOutput;
                    use $crate::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<$cls>::new();
                    if let Some(value) = ::std::ptr::NonNull::new(value) {
                        unsafe { collector.$set(py, _slf, attr, value).convert(py) }
                    } else {
                        unsafe { collector.$del(py, _slf, attr).convert(py) }
                    }
                }

                $crate::ffi::PyType_Slot {
                    slot: $crate::ffi::$slot,
                    pfunc: $crate::impl_::trampoline::get_trampoline_function!(
                        setattrofunc,
                        slot_impl
                    ) as $crate::ffi::$func_ty as _,
                }
            }};
        }
        pub use $generate_macro;
    };
}

define_pyclass_setattr_slot! {
    PyClass__setattr__SlotFragment,
    PyClass__delattr__SlotFragment,
    __setattr__,
    __delattr__,
    Err(PyAttributeError::new_err("can't set attribute")),
    Err(PyAttributeError::new_err("can't delete attribute")),
    generate_pyclass_setattr_slot,
    Py_tp_setattro,
    setattrofunc,
}

define_pyclass_setattr_slot! {
    PyClass__set__SlotFragment,
    PyClass__delete__SlotFragment,
    __set__,
    __delete__,
    Err(PyNotImplementedError::new_err("can't set descriptor")),
    Err(PyNotImplementedError::new_err("can't delete descriptor")),
    generate_pyclass_setdescr_slot,
    Py_tp_descr_set,
    descrsetfunc,
}

define_pyclass_setattr_slot! {
    PyClass__setitem__SlotFragment,
    PyClass__delitem__SlotFragment,
    __setitem__,
    __delitem__,
    Err(PyNotImplementedError::new_err("can't set item")),
    Err(PyNotImplementedError::new_err("can't delete item")),
    generate_pyclass_setitem_slot,
    Py_mp_ass_subscript,
    objobjargproc,
}

/// Macro which expands to three items
/// - Trait for a lhs dunder e.g. __add__
/// - Trait for the corresponding rhs e.g. __radd__
/// - A macro which will use dtolnay specialisation to generate the shared slot for the two dunders
macro_rules! define_pyclass_binary_operator_slot {
    (
        $lhs_trait:ident,
        $rhs_trait:ident,
        $lhs:ident,
        $rhs:ident,
        $generate_macro:ident,
        $slot:ident,
    ) => {
        slot_fragment_trait! {
            $lhs_trait,

            /// # Safety: _slf and _other must be valid non-null Python objects
            #[inline]
            unsafe fn $lhs(
                self,
                py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _other: *mut ffi::PyObject,
            ) -> PyResult<*mut ffi::PyObject> {
                Ok(py.NotImplemented().into_ptr())
            }
        }

        slot_fragment_trait! {
            $rhs_trait,

            /// # Safety: _slf and _other must be valid non-null Python objects
            #[inline]
            unsafe fn $rhs(
                self,
                py: Python<'_>,
                _slf: *mut ffi::PyObject,
                _other: *mut ffi::PyObject,
            ) -> PyResult<*mut ffi::PyObject> {
                Ok(py.NotImplemented().into_ptr())
            }
        }

        #[doc(hidden)]
        #[macro_export]
        macro_rules! $generate_macro {
            ($cls:ty) => {{
                unsafe fn slot_impl(
                    py: $crate::Python<'_>,
                    _slf: *mut $crate::ffi::PyObject,
                    _other: *mut $crate::ffi::PyObject,
                ) -> $crate::PyResult<*mut $crate::ffi::PyObject> {
                    use $crate::impl_::pyclass::*;
                    let collector = PyClassImplCollector::<$cls>::new();
                    let lhs_result = unsafe { collector.$lhs(py, _slf, _other) }?;
                    if lhs_result == unsafe { $crate::ffi::Py_NotImplemented() } {
                        unsafe { $crate::ffi::Py_DECREF(lhs_result) };
                        unsafe { collector.$rhs(py, _other, _slf) }
                    } else {
                        ::std::result::Result::Ok(lhs_result)
                    }
                }

                $crate::ffi::PyType_Slot {
                    slot: $crate::ffi::$slot,
                    pfunc: $crate::impl_::trampoline::get_trampoline_function!(
                        binaryfunc, slot_impl
                    ) as $crate::ffi::binaryfunc as _,
                }
            }};
        }
        pub use $generate_macro;
    };
}

define_pyclass_binary_operator_slot! {
    PyClass__add__SlotFragment,
    PyClass__radd__SlotFragment,
    __add__,
    __radd__,
    generate_pyclass_add_slot,
    Py_nb_add,
}

define_pyclass_binary_operator_slot! {
    PyClass__sub__SlotFragment,
    PyClass__rsub__SlotFragment,
    __sub__,
    __rsub__,
    generate_pyclass_sub_slot,
    Py_nb_subtract,
}

define_pyclass_binary_operator_slot! {
    PyClass__mul__SlotFragment,
    PyClass__rmul__SlotFragment,
    __mul__,
    __rmul__,
    generate_pyclass_mul_slot,
    Py_nb_multiply,
}

define_pyclass_binary_operator_slot! {
    PyClass__mod__SlotFragment,
    PyClass__rmod__SlotFragment,
    __mod__,
    __rmod__,
    generate_pyclass_mod_slot,
    Py_nb_remainder,
}

define_pyclass_binary_operator_slot! {
    PyClass__divmod__SlotFragment,
    PyClass__rdivmod__SlotFragment,
    __divmod__,
    __rdivmod__,
    generate_pyclass_divmod_slot,
    Py_nb_divmod,
}

define_pyclass_binary_operator_slot! {
    PyClass__lshift__SlotFragment,
    PyClass__rlshift__SlotFragment,
    __lshift__,
    __rlshift__,
    generate_pyclass_lshift_slot,
    Py_nb_lshift,
}

define_pyclass_binary_operator_slot! {
    PyClass__rshift__SlotFragment,
    PyClass__rrshift__SlotFragment,
    __rshift__,
    __rrshift__,
    generate_pyclass_rshift_slot,
    Py_nb_rshift,
}

define_pyclass_binary_operator_slot! {
    PyClass__and__SlotFragment,
    PyClass__rand__SlotFragment,
    __and__,
    __rand__,
    generate_pyclass_and_slot,
    Py_nb_and,
}

define_pyclass_binary_operator_slot! {
    PyClass__or__SlotFragment,
    PyClass__ror__SlotFragment,
    __or__,
    __ror__,
    generate_pyclass_or_slot,
    Py_nb_or,
}

define_pyclass_binary_operator_slot! {
    PyClass__xor__SlotFragment,
    PyClass__rxor__SlotFragment,
    __xor__,
    __rxor__,
    generate_pyclass_xor_slot,
    Py_nb_xor,
}

define_pyclass_binary_operator_slot! {
    PyClass__matmul__SlotFragment,
    PyClass__rmatmul__SlotFragment,
    __matmul__,
    __rmatmul__,
    generate_pyclass_matmul_slot,
    Py_nb_matrix_multiply,
}

define_pyclass_binary_operator_slot! {
    PyClass__truediv__SlotFragment,
    PyClass__rtruediv__SlotFragment,
    __truediv__,
    __rtruediv__,
    generate_pyclass_truediv_slot,
    Py_nb_true_divide,
}

define_pyclass_binary_operator_slot! {
    PyClass__floordiv__SlotFragment,
    PyClass__rfloordiv__SlotFragment,
    __floordiv__,
    __rfloordiv__,
    generate_pyclass_floordiv_slot,
    Py_nb_floor_divide,
}

slot_fragment_trait! {
    PyClass__pow__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __pow__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
        _mod: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__rpow__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __rpow__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
        _mod: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_pow_slot {
    ($cls:ty) => {{
        fn slot_impl(
            py: $crate::Python<'_>,
            _slf: *mut $crate::ffi::PyObject,
            _other: *mut $crate::ffi::PyObject,
            _mod: *mut $crate::ffi::PyObject,
        ) -> $crate::PyResult<*mut $crate::ffi::PyObject> {
            use $crate::impl_::pyclass::*;
            let collector = PyClassImplCollector::<$cls>::new();
            let lhs_result = unsafe { collector.__pow__(py, _slf, _other, _mod) }?;
            if lhs_result == unsafe { $crate::ffi::Py_NotImplemented() } {
                unsafe { $crate::ffi::Py_DECREF(lhs_result) };
                unsafe { collector.__rpow__(py, _other, _slf, _mod) }
            } else {
                ::std::result::Result::Ok(lhs_result)
            }
        }

        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_nb_power,
            pfunc: $crate::impl_::trampoline::get_trampoline_function!(ternaryfunc, slot_impl)
                as $crate::ffi::ternaryfunc as _,
        }
    }};
}
pub use generate_pyclass_pow_slot;

slot_fragment_trait! {
    PyClass__lt__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __lt__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__le__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __le__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__eq__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __eq__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__ne__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __ne__(
        self,
        py: Python<'_>,
        slf: *mut ffi::PyObject,
        other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        // By default `__ne__` will try `__eq__` and invert the result
        let slf = unsafe { Borrowed::from_ptr(py, slf)};
        let other = unsafe { Borrowed::from_ptr(py, other)};
        slf.eq(other).map(|is_eq| PyBool::new(py, !is_eq).to_owned().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__gt__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __gt__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

slot_fragment_trait! {
    PyClass__ge__SlotFragment,

    /// # Safety: _slf and _other must be valid non-null Python objects
    #[inline]
    unsafe fn __ge__(
        self,
        py: Python<'_>,
        _slf: *mut ffi::PyObject,
        _other: *mut ffi::PyObject,
    ) -> PyResult<*mut ffi::PyObject> {
        Ok(py.NotImplemented().into_ptr())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_pyclass_richcompare_slot {
    ($cls:ty) => {{
        #[allow(unknown_lints, non_local_definitions)]
        impl $cls {
            #[expect(non_snake_case)]
            unsafe fn __pymethod___richcmp____(
                py: $crate::Python<'_>,
                slf: *mut $crate::ffi::PyObject,
                other: *mut $crate::ffi::PyObject,
                op: ::std::ffi::c_int,
            ) -> $crate::PyResult<*mut $crate::ffi::PyObject> {
                use $crate::class::basic::CompareOp;
                use $crate::impl_::pyclass::*;
                let collector = PyClassImplCollector::<$cls>::new();
                match CompareOp::from_raw(op).expect("invalid compareop") {
                    CompareOp::Lt => unsafe { collector.__lt__(py, slf, other) },
                    CompareOp::Le => unsafe { collector.__le__(py, slf, other) },
                    CompareOp::Eq => unsafe { collector.__eq__(py, slf, other) },
                    CompareOp::Ne => unsafe { collector.__ne__(py, slf, other) },
                    CompareOp::Gt => unsafe { collector.__gt__(py, slf, other) },
                    CompareOp::Ge => unsafe { collector.__ge__(py, slf, other) },
                }
            }
        }
        $crate::ffi::PyType_Slot {
            slot: $crate::ffi::Py_tp_richcompare,
            pfunc: {
                type Cls = $cls; // `get_trampoline_function` doesn't accept $cls directly
                $crate::impl_::trampoline::get_trampoline_function!(
                    richcmpfunc,
                    Cls::__pymethod___richcmp____
                ) as $crate::ffi::richcmpfunc as _
            },
        }
    }};
}
pub use generate_pyclass_richcompare_slot;

/// Implements a freelist.
///
/// Do not implement this trait manually. Instead, use `#[pyclass(freelist = N)]`
/// on a Rust struct to implement it.
pub trait PyClassWithFreeList: PyClass {
    fn get_free_list(py: Python<'_>) -> &'static Mutex<PyObjectFreeList>;
}

/// Implementation of tp_alloc for `freelist` classes.
///
/// # Safety
/// - `subtype` must be a valid pointer to the type object of T or a subclass.
/// - The calling thread must be attached to the interpreter
pub unsafe extern "C" fn alloc_with_freelist<T: PyClassWithFreeList>(
    subtype: *mut ffi::PyTypeObject,
    nitems: ffi::Py_ssize_t,
) -> *mut ffi::PyObject {
    let py = unsafe { Python::assume_attached() };

    #[cfg(not(Py_3_8))]
    unsafe {
        bpo_35810_workaround(py, subtype)
    };

    let self_type = T::type_object_raw(py);
    // If this type is a variable type or the subtype is not equal to this type, we cannot use the
    // freelist
    if nitems == 0 && ptr::eq(subtype, self_type) {
        let mut free_list = T::get_free_list(py).lock().unwrap();
        if let Some(obj) = free_list.pop() {
            drop(free_list);
            unsafe { ffi::PyObject_Init(obj, subtype) };
            unsafe { ffi::PyObject_Init(obj, subtype) };
            return obj as _;
        }
    }

    unsafe { ffi::PyType_GenericAlloc(subtype, nitems) }
}

/// Implementation of tp_free for `freelist` classes.
///
/// # Safety
/// - `obj` must be a valid pointer to an instance of T (not a subclass).
/// - The calling thread must be attached to the interpreter
pub unsafe extern "C" fn free_with_freelist<T: PyClassWithFreeList>(obj: *mut c_void) {
    let obj = obj as *mut ffi::PyObject;
    unsafe {
        debug_assert_eq!(
            T::type_object_raw(Python::assume_attached()),
            ffi::Py_TYPE(obj)
        );
        let mut free_list = T::get_free_list(Python::assume_attached()).lock().unwrap();
        if let Some(obj) = free_list.insert(obj) {
            drop(free_list);
            let ty = ffi::Py_TYPE(obj);

            // Deduce appropriate inverse of PyType_GenericAlloc
            let free = if ffi::PyType_IS_GC(ty) != 0 {
                ffi::PyObject_GC_Del
            } else {
                ffi::PyObject_Free
            };
            free(obj as *mut c_void);

            #[cfg(Py_3_8)]
            if ffi::PyType_HasFeature(ty, ffi::Py_TPFLAGS_HEAPTYPE) != 0 {
                ffi::Py_DECREF(ty as *mut ffi::PyObject);
            }
        }
    }
}

/// Workaround for Python issue 35810; no longer necessary in Python 3.8
#[inline]
#[cfg(not(Py_3_8))]
unsafe fn bpo_35810_workaround(py: Python<'_>, ty: *mut ffi::PyTypeObject) {
    #[cfg(Py_LIMITED_API)]
    {
        // Must check version at runtime for abi3 wheels - they could run against a higher version
        // than the build config suggests.
        use crate::sync::PyOnceLock;
        static IS_PYTHON_3_8: PyOnceLock<bool> = PyOnceLock::new();

        if *IS_PYTHON_3_8.get_or_init(py, || py.version_info() >= (3, 8)) {
            // No fix needed - the wheel is running on a sufficiently new interpreter.
            return;
        }
    }
    #[cfg(not(Py_LIMITED_API))]
    {
        // suppress unused variable warning
        let _ = py;
    }

    unsafe { ffi::Py_INCREF(ty as *mut ffi::PyObject) };
}

/// Method storage for `#[pyclass]`.
///
/// Implementation detail. Only to be used through our proc macro code.
/// Allows arbitrary `#[pymethod]` blocks to submit their methods,
/// which are eventually collected by `#[pyclass]`.
#[cfg(feature = "multiple-pymethods")]
pub trait PyClassInventory: inventory::Collect {
    /// Returns the items for a single `#[pymethods] impl` block
    fn items(&'static self) -> &'static PyClassItems;
}

// Items from #[pymethods] if not using inventory.
#[cfg(not(feature = "multiple-pymethods"))]
pub trait PyMethods<T> {
    fn py_methods(self) -> &'static PyClassItems;
}

#[cfg(not(feature = "multiple-pymethods"))]
impl<T> PyMethods<T> for &'_ PyClassImplCollector<T> {
    fn py_methods(self) -> &'static PyClassItems {
        &PyClassItems {
            methods: &[],
            slots: &[],
        }
    }
}

// Thread checkers

#[doc(hidden)]
pub trait PyClassThreadChecker<T>: Sized + sealed::Sealed {
    fn ensure(&self);
    fn check(&self) -> bool;
    fn can_drop(&self, py: Python<'_>) -> bool;
    fn new() -> Self;
}

/// Default thread checker for `#[pyclass]`.
#[doc(hidden)]
pub struct NoopThreadChecker;

impl<T> PyClassThreadChecker<T> for NoopThreadChecker {
    fn ensure(&self) {}
    fn check(&self) -> bool {
        true
    }
    fn can_drop(&self, _py: Python<'_>) -> bool {
        true
    }
    #[inline]
    fn new() -> Self {
        NoopThreadChecker
    }
}

/// Thread checker for `#[pyclass(unsendable)]` types.
/// Panics when the value is accessed by another thread.
#[doc(hidden)]
pub struct ThreadCheckerImpl(thread::ThreadId);

impl ThreadCheckerImpl {
    fn ensure(&self, type_name: &'static str) {
        assert_eq!(
            thread::current().id(),
            self.0,
            "{type_name} is unsendable, but sent to another thread"
        );
    }

    fn check(&self) -> bool {
        thread::current().id() == self.0
    }

    fn can_drop(&self, py: Python<'_>, type_name: &'static str) -> bool {
        if thread::current().id() != self.0 {
            PyRuntimeError::new_err(format!(
                "{type_name} is unsendable, but is being dropped on another thread"
            ))
            .write_unraisable(py, None);
            return false;
        }

        true
    }
}

impl<T> PyClassThreadChecker<T> for ThreadCheckerImpl {
    fn ensure(&self) {
        self.ensure(std::any::type_name::<T>());
    }
    fn check(&self) -> bool {
        self.check()
    }
    fn can_drop(&self, py: Python<'_>) -> bool {
        self.can_drop(py, std::any::type_name::<T>())
    }
    fn new() -> Self {
        ThreadCheckerImpl(thread::current().id())
    }
}

/// Trait denoting that this class is suitable to be used as a base type for PyClass.
#[cfg_attr(
    Py_LIMITED_API,
    diagnostic::on_unimplemented(
        message = "pyclass `{Self}` cannot be subclassed",
        label = "required for `#[pyclass(extends={Self})]`",
        note = "`{Self}` must have `#[pyclass(subclass)]` to be eligible for subclassing",
        note = "with the `abi3` feature enabled, PyO3 does not support subclassing native types",
    )
)]
#[cfg_attr(
    not(Py_LIMITED_API),
    diagnostic::on_unimplemented(
        message = "pyclass `{Self}` cannot be subclassed",
        label = "required for `#[pyclass(extends={Self})]`",
        note = "`{Self}` must have `#[pyclass(subclass)]` to be eligible for subclassing",
    )
)]
pub trait PyClassBaseType: Sized {
    type LayoutAsBase: PyClassObjectBaseLayout<Self>;
    type BaseNativeType;
    type Initializer: PyObjectInit<Self>;
    type PyClassMutability: PyClassMutability;
    /// The type of object layout to use for ancestors or descendants of this type.
    type Layout<T: PyClassImpl>;
}

/// Implementation of tp_dealloc for pyclasses without gc
pub(crate) unsafe extern "C" fn tp_dealloc<T: PyClass>(obj: *mut ffi::PyObject) {
    unsafe { crate::impl_::trampoline::dealloc(obj, <T as PyClassImpl>::Layout::tp_dealloc) }
}

/// Implementation of tp_dealloc for pyclasses with gc
pub(crate) unsafe extern "C" fn tp_dealloc_with_gc<T: PyClass>(obj: *mut ffi::PyObject) {
    #[cfg(not(PyPy))]
    unsafe {
        ffi::PyObject_GC_UnTrack(obj.cast());
    }
    unsafe { crate::impl_::trampoline::dealloc(obj, <T as PyClassImpl>::Layout::tp_dealloc) }
}

pub(crate) unsafe extern "C" fn get_sequence_item_from_mapping(
    obj: *mut ffi::PyObject,
    index: ffi::Py_ssize_t,
) -> *mut ffi::PyObject {
    let index = unsafe { ffi::PyLong_FromSsize_t(index) };
    if index.is_null() {
        return std::ptr::null_mut();
    }
    let result = unsafe { ffi::PyObject_GetItem(obj, index) };
    unsafe { ffi::Py_DECREF(index) };
    result
}

pub(crate) unsafe extern "C" fn assign_sequence_item_from_mapping(
    obj: *mut ffi::PyObject,
    index: ffi::Py_ssize_t,
    value: *mut ffi::PyObject,
) -> c_int {
    unsafe {
        let index = ffi::PyLong_FromSsize_t(index);
        if index.is_null() {
            return -1;
        }
        let result = if value.is_null() {
            ffi::PyObject_DelItem(obj, index)
        } else {
            ffi::PyObject_SetItem(obj, index, value)
        };
        ffi::Py_DECREF(index);
        result
    }
}

/// Offset of a field within a PyObject in bytes.
#[derive(Debug, Clone, Copy)]
pub enum PyObjectOffset {
    /// An offset relative to the start of the object
    Absolute(ffi::Py_ssize_t),
    /// An offset relative to the start of the subclass-specific data.
    /// Only allowed when basicsize is negative (which is only allowed for python >=3.12).
    /// <https://docs.python.org/3.12/c-api/structures.html#c.Py_RELATIVE_OFFSET>
    #[cfg(Py_3_12)]
    Relative(ffi::Py_ssize_t),
}

impl std::ops::Add<usize> for PyObjectOffset {
    type Output = PyObjectOffset;

    fn add(self, rhs: usize) -> Self::Output {
        // Py_ssize_t may not be equal to isize on all platforms
        #[allow(clippy::useless_conversion)]
        let rhs: ffi::Py_ssize_t = rhs.try_into().expect("offset should fit in Py_ssize_t");

        match self {
            PyObjectOffset::Absolute(offset) => PyObjectOffset::Absolute(offset + rhs),
            #[cfg(Py_3_12)]
            PyObjectOffset::Relative(offset) => PyObjectOffset::Relative(offset + rhs),
        }
    }
}

/// Type which uses specialization on impl blocks to determine how to read a field from a Rust pyclass
/// as part of a `#[pyo3(get)]` annotation.
pub struct PyClassGetterGenerator<
    // structural information about the field: class type, field type, offset of the field within
    // the class struct
    ClassT: PyClass,
    FieldT,
    const OFFSET: usize,
    // additional metadata about the field which is used to switch between different implementations
    // at compile time
    const IS_PY_T: bool,
    const IMPLEMENTS_INTOPYOBJECT_REF: bool,
    const IMPLEMENTS_INTOPYOBJECT: bool,
>(PhantomData<(ClassT, FieldT)>);

impl<
        ClassT: PyClass,
        FieldT,
        const OFFSET: usize,
        const IS_PY_T: bool,
        const IMPLEMENTS_INTOPYOBJECT_REF: bool,
        const IMPLEMENTS_INTOPYOBJECT: bool,
    >
    PyClassGetterGenerator<
        ClassT,
        FieldT,
        OFFSET,
        IS_PY_T,
        IMPLEMENTS_INTOPYOBJECT_REF,
        IMPLEMENTS_INTOPYOBJECT,
    >
{
    /// Safety: constructing this type requires that there exists a value of type FieldT
    /// at the calculated offset within the type ClassT.
    pub const unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<
        ClassT: PyClass,
        U: PyTypeCheck,
        const OFFSET: usize,
        const IMPLEMENTS_INTOPYOBJECT_REF: bool,
        const IMPLEMENTS_INTOPYOBJECT: bool,
    >
    PyClassGetterGenerator<
        ClassT,
        Py<U>,
        OFFSET,
        true,
        IMPLEMENTS_INTOPYOBJECT_REF,
        IMPLEMENTS_INTOPYOBJECT,
    >
{
    /// `Py<T>` fields have a potential optimization to use Python's "struct members" to read
    /// the field directly from the struct, rather than using a getter function.
    ///
    /// This is the most efficient operation the Python interpreter could possibly do to
    /// read a field, but it's only possible for us to allow this for frozen classes.
    pub const fn generate(&self, name: &'static CStr, doc: &'static CStr) -> PyMethodDefType {
        use crate::pyclass::boolean_struct::private::Boolean;
        if ClassT::Frozen::VALUE {
            let (offset, flags) = match <ClassT as PyClassImpl>::Layout::CONTENTS_OFFSET {
                PyObjectOffset::Absolute(offset) => (offset, ffi::Py_READONLY),
                #[cfg(Py_3_12)]
                PyObjectOffset::Relative(offset) => {
                    (offset, ffi::Py_READONLY | ffi::Py_RELATIVE_OFFSET)
                }
            };

            PyMethodDefType::StructMember(ffi::PyMemberDef {
                name: name.as_ptr(),
                type_code: ffi::Py_T_OBJECT_EX,
                offset: offset + OFFSET as ffi::Py_ssize_t,
                flags,
                doc: doc.as_ptr(),
            })
        } else {
            PyMethodDefType::Getter(PyGetterDef {
                name,
                meth: pyo3_get_value_into_pyobject_ref::<ClassT, Py<U>, OFFSET>,
                doc,
            })
        }
    }
}

/// Field is not `Py<T>`; try to use `IntoPyObject` for `&T` (preferred over `ToPyObject`) to avoid
/// potentially expensive clones of containers like `Vec`
impl<ClassT, FieldT, const OFFSET: usize, const IMPLEMENTS_INTOPYOBJECT: bool>
    PyClassGetterGenerator<ClassT, FieldT, OFFSET, false, true, IMPLEMENTS_INTOPYOBJECT>
where
    ClassT: PyClass,
    for<'a, 'py> &'a FieldT: IntoPyObject<'py>,
{
    pub const fn generate(&self, name: &'static CStr, doc: &'static CStr) -> PyMethodDefType {
        PyMethodDefType::Getter(PyGetterDef {
            name,
            meth: pyo3_get_value_into_pyobject_ref::<ClassT, FieldT, OFFSET>,
            doc,
        })
    }
}

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be converted to a Python object",
    label = "required by `#[pyo3(get)]` to create a readable property from a field of type `{Self}`",
    note = "implement `IntoPyObject` for `&{Self}` or `IntoPyObject + Clone` for `{Self}` to define the conversion"
)]
pub trait PyO3GetField<'py>: IntoPyObject<'py> + Clone {}
impl<'py, T> PyO3GetField<'py> for T where T: IntoPyObject<'py> + Clone {}

/// Base case attempts to use IntoPyObject + Clone
impl<ClassT: PyClass, FieldT, const OFFSET: usize, const IMPLEMENTS_INTOPYOBJECT: bool>
    PyClassGetterGenerator<ClassT, FieldT, OFFSET, false, false, IMPLEMENTS_INTOPYOBJECT>
{
    pub const fn generate(&self, name: &'static CStr, doc: &'static CStr) -> PyMethodDefType
    // The bound goes here rather than on the block so that this impl is always available
    // if no specialization is used instead
    where
        for<'py> FieldT: PyO3GetField<'py>,
    {
        PyMethodDefType::Getter(PyGetterDef {
            name,
            meth: pyo3_get_value_into_pyobject::<ClassT, FieldT, OFFSET>,
            doc,
        })
    }
}

/// ensures `obj` is not mutably aliased
#[inline]
unsafe fn ensure_no_mutable_alias<'a, ClassT: PyClass>(
    _py: Python<'_>,
    obj: &'a *mut ffi::PyObject,
) -> Result<PyClassGuard<'a, ClassT>, PyBorrowError> {
    unsafe { PyClassGuard::try_borrow(NonNull::from(obj).cast::<Py<ClassT>>().as_ref()) }
}

/// Gets a field value from a pyclass and produces a python value using `IntoPyObject` for `&FieldT`
///
/// # Safety
/// - `obj` must be a valid pointer to an instance of `ClassT`
/// - there must be a value of type `FieldT` at the calculated offset within `ClassT`
unsafe fn pyo3_get_value_into_pyobject_ref<ClassT, FieldT, const OFFSET: usize>(
    py: Python<'_>,
    obj: *mut ffi::PyObject,
) -> PyResult<*mut ffi::PyObject>
where
    ClassT: PyClass,
    for<'a, 'py> &'a FieldT: IntoPyObject<'py>,
{
    /// Inner function to convert the field value at the given offset
    ///
    /// # Safety
    /// - mutable aliasing is prevented by the caller
    /// - value of type `FieldT` must exist at the given offset within obj
    unsafe fn inner<FieldT>(
        py: Python<'_>,
        obj: *const (),
        offset: usize,
    ) -> PyResult<*mut ffi::PyObject>
    where
        for<'a, 'py> &'a FieldT: IntoPyObject<'py>,
    {
        // SAFETY: caller upholds safety invariants
        let value = unsafe { &*obj.byte_add(offset).cast::<FieldT>() };
        value.into_py_any(py).map(Py::into_ptr)
    }

    // SAFETY: `obj` is a valid pointer to `ClassT`
    let _holder = unsafe { ensure_no_mutable_alias::<ClassT>(py, &obj)? };
    let class_ptr = obj.cast::<<ClassT as PyClassImpl>::Layout>();
    let class_obj = unsafe { &*class_ptr };
    let contents_ptr = ptr::from_ref(class_obj.contents());

    // SAFETY: _holder prevents mutable aliasing, caller upholds other safety invariants
    unsafe { inner::<FieldT>(py, contents_ptr.cast(), OFFSET) }
}

/// Gets a field value from a pyclass and produces a python value using `IntoPyObject` for `FieldT`,
/// after cloning the value.
///
/// # Safety
/// - `obj` must be a valid pointer to an instance of `ClassT`
/// - there must be a value of type `FieldT` at the calculated offset within `ClassT`
unsafe fn pyo3_get_value_into_pyobject<ClassT, FieldT, const OFFSET: usize>(
    py: Python<'_>,
    obj: *mut ffi::PyObject,
) -> PyResult<*mut ffi::PyObject>
where
    ClassT: PyClass,
    for<'py> FieldT: IntoPyObject<'py> + Clone,
{
    /// Inner function to convert the field value at the given offset
    ///
    /// # Safety
    /// - mutable aliasing is prevented by the caller
    /// - value of type `FieldT` must exist at the given offset within obj
    unsafe fn inner<FieldT>(
        py: Python<'_>,
        obj: *const (),
        offset: usize,
    ) -> PyResult<*mut ffi::PyObject>
    where
        for<'py> FieldT: IntoPyObject<'py> + Clone,
    {
        // SAFETY: caller upholds safety invariants
        let value = unsafe { &*obj.byte_add(offset).cast::<FieldT>() };
        value.clone().into_py_any(py).map(Py::into_ptr)
    }

    // SAFETY: `obj` is a valid pointer to `ClassT`
    let _holder = unsafe { ensure_no_mutable_alias::<ClassT>(py, &obj)? };
    let class_ptr = obj.cast::<<ClassT as PyClassImpl>::Layout>();
    let class_obj = unsafe { &*class_ptr };
    let contents_ptr = ptr::from_ref(class_obj.contents());

    // SAFETY: _holder prevents mutable aliasing, caller upholds other safety invariants
    unsafe { inner::<FieldT>(py, contents_ptr.cast(), OFFSET) }
}

pub struct ConvertField<
    const IMPLEMENTS_INTOPYOBJECT_REF: bool,
    const IMPLEMENTS_INTOPYOBJECT: bool,
>;

impl<const IMPLEMENTS_INTOPYOBJECT: bool> ConvertField<true, IMPLEMENTS_INTOPYOBJECT> {
    #[inline]
    pub fn convert_field<'a, 'py, T>(obj: &'a T, py: Python<'py>) -> PyResult<Py<PyAny>>
    where
        &'a T: IntoPyObject<'py>,
    {
        obj.into_py_any(py)
    }
}

impl<const IMPLEMENTS_INTOPYOBJECT: bool> ConvertField<false, IMPLEMENTS_INTOPYOBJECT> {
    #[inline]
    pub fn convert_field<'py, T>(obj: &T, py: Python<'py>) -> PyResult<Py<PyAny>>
    where
        T: PyO3GetField<'py>,
    {
        obj.clone().into_py_any(py)
    }
}

pub trait ExtractPyClassWithClone {}

#[cfg(test)]
#[cfg(feature = "macros")]
mod tests {
    use crate::pycell::impl_::PyClassObjectContents;

    use super::*;
    use std::mem::offset_of;

    #[test]
    fn get_py_for_frozen_class() {
        #[crate::pyclass(crate = "crate", frozen)]
        struct FrozenClass {
            #[pyo3(get)]
            value: Py<PyAny>,
        }

        let mut methods = Vec::new();
        let mut slots = Vec::new();

        for items in FrozenClass::items_iter() {
            methods.extend_from_slice(items.methods);
            slots.extend_from_slice(items.slots);
        }

        assert_eq!(methods.len(), 1);
        assert!(slots.is_empty());

        match methods.first() {
            Some(PyMethodDefType::StructMember(member)) => {
                assert_eq!(unsafe { CStr::from_ptr(member.name) }, c"value");
                assert_eq!(member.type_code, ffi::Py_T_OBJECT_EX);
                #[repr(C)]
                struct ExpectedLayout {
                    ob_base: ffi::PyObject,
                    contents: PyClassObjectContents<FrozenClass>,
                }
                assert_eq!(
                    member.offset,
                    (offset_of!(ExpectedLayout, contents) + offset_of!(FrozenClass, value))
                        as ffi::Py_ssize_t
                );
                assert_eq!(member.flags, ffi::Py_READONLY);
            }
            _ => panic!("Expected a StructMember"),
        }
    }

    #[test]
    fn get_py_for_non_frozen_class() {
        #[crate::pyclass(crate = "crate")]
        struct FrozenClass {
            #[pyo3(get)]
            value: Py<PyAny>,
        }

        let mut methods = Vec::new();
        let mut slots = Vec::new();

        for items in FrozenClass::items_iter() {
            methods.extend_from_slice(items.methods);
            slots.extend_from_slice(items.slots);
        }

        assert_eq!(methods.len(), 1);
        assert!(slots.is_empty());

        match methods.first() {
            Some(PyMethodDefType::Getter(getter)) => {
                assert_eq!(getter.name, c"value");
                assert_eq!(getter.doc, c"");
                // tests for the function pointer are in test_getter_setter.py
            }
            _ => panic!("Expected a StructMember"),
        }
    }

    #[test]
    fn test_field_getter_generator() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {
            my_field: i32,
        }

        const FIELD_OFFSET: usize = offset_of!(MyClass, my_field);

        // generate for a non-py field using IntoPyObject for &i32
        // SAFETY: offset is correct
        let generator = unsafe {
            PyClassGetterGenerator::<MyClass, i32, FIELD_OFFSET, false, true, false>::new()
        };
        let PyMethodDefType::Getter(def) = generator.generate(c"my_field", c"My field doc") else {
            panic!("Expected a Getter");
        };

        assert_eq!(def.name, c"my_field");
        assert_eq!(def.doc, c"My field doc");

        #[cfg(fn_ptr_eq)]
        {
            use crate::impl_::pymethods::Getter;

            assert!(std::ptr::fn_addr_eq(
                def.meth,
                pyo3_get_value_into_pyobject_ref::<MyClass, i32, FIELD_OFFSET> as Getter
            ));
        }

        // generate for a field via `IntoPyObject` + `Clone`
        // SAFETY: offset is correct
        let generator = unsafe {
            PyClassGetterGenerator::<MyClass, String, FIELD_OFFSET, false, false, true>::new()
        };
        let PyMethodDefType::Getter(def) = generator.generate(c"my_field", c"My field doc") else {
            panic!("Expected a Getter");
        };
        assert_eq!(def.name, c"my_field");
        assert_eq!(def.doc, c"My field doc");

        #[cfg(fn_ptr_eq)]
        {
            use crate::impl_::pymethods::Getter;

            assert!(std::ptr::fn_addr_eq(
                def.meth,
                pyo3_get_value_into_pyobject::<MyClass, String, FIELD_OFFSET> as Getter
            ));
        }
    }

    #[test]
    fn test_field_getter_generator_py_field_frozen() {
        #[crate::pyclass(crate = "crate", frozen)]
        struct MyClass {
            my_field: Py<PyAny>,
        }

        const FIELD_OFFSET: usize = offset_of!(MyClass, my_field);
        // SAFETY: offset is correct
        let generator = unsafe {
            PyClassGetterGenerator::<MyClass, Py<PyAny>, FIELD_OFFSET, true, true, true>::new()
        };
        let PyMethodDefType::StructMember(def) = generator.generate(c"my_field", c"My field doc")
        else {
            panic!("Expected a StructMember");
        };
        // SAFETY: def.name originated from a CStr
        assert_eq!(unsafe { CStr::from_ptr(def.name) }, c"my_field");
        // SAFETY: def.doc originated from a CStr
        assert_eq!(unsafe { CStr::from_ptr(def.doc) }, c"My field doc");
        assert_eq!(def.type_code, ffi::Py_T_OBJECT_EX);
        #[allow(irrefutable_let_patterns)]
        let PyObjectOffset::Absolute(contents_offset) =
            <MyClass as PyClassImpl>::Layout::CONTENTS_OFFSET
        else {
            panic!()
        };
        assert_eq!(
            def.offset,
            contents_offset + FIELD_OFFSET as ffi::Py_ssize_t
        );
        assert_eq!(def.flags, ffi::Py_READONLY);
    }

    #[test]
    fn test_field_getter_generator_py_field_non_frozen() {
        #[crate::pyclass(crate = "crate")]
        struct MyClass {
            my_field: Py<PyAny>,
        }

        const FIELD_OFFSET: usize = offset_of!(MyClass, my_field);
        // SAFETY: offset is correct
        let generator = unsafe {
            PyClassGetterGenerator::<MyClass, Py<PyAny>, FIELD_OFFSET, true, true, true>::new()
        };
        let PyMethodDefType::Getter(def) = generator.generate(c"my_field", c"My field doc") else {
            panic!("Expected a Getter");
        };
        assert_eq!(def.name, c"my_field");
        assert_eq!(def.doc, c"My field doc");

        #[cfg(fn_ptr_eq)]
        {
            use crate::impl_::pymethods::Getter;

            assert!(std::ptr::fn_addr_eq(
                def.meth,
                pyo3_get_value_into_pyobject_ref::<MyClass, Py<PyAny>, FIELD_OFFSET> as Getter
            ));
        }
    }
}
