//! `PyClass` and related traits.
use crate::{
    callback::IntoPyCallbackOutput,
    exceptions::PyTypeError,
    ffi,
    impl_::pyclass::{
        assign_sequence_item_from_mapping, get_sequence_item_from_mapping, tp_dealloc, PyClassImpl,
        PyClassItemsIter,
    },
    IntoPy, IntoPyPointer, PyCell, PyErr, PyMethodDefType, PyObject, PyResult, PyTypeInfo, Python,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    convert::TryInto,
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_ulong, c_void},
    ptr,
};

/// Types that can be used as Python classes.
///
/// The `#[pyclass]` attribute implements this trait for your Rust struct -
/// you shouldn't implement this trait directly.
pub trait PyClass:
    PyTypeInfo<AsRefTarget = PyCell<Self>> + PyClassImpl<Layout = PyCell<Self>>
{
    /// Whether the pyclass is frozen.
    ///
    /// This can be enabled via `#[pyclass(frozen)]`.
    type Frozen: Frozen;
}

pub(crate) fn create_type_object<T>(py: Python<'_>) -> *mut ffi::PyTypeObject
where
    T: PyClass,
{
    match unsafe {
        PyTypeBuilder::default()
            .with_type_doc(T::DOC)
            .with_offsets(T::dict_offset(), T::weaklist_offset())
            .with_slot(ffi::Py_tp_base, T::BaseType::type_object_raw(py))
            .with_slot(ffi::Py_tp_dealloc, tp_dealloc::<T> as *mut c_void)
            .set_is_basetype(T::IS_BASETYPE)
            .set_is_mapping(T::IS_MAPPING)
            .with_class_items(T::items_iter())
            .build(py, T::NAME, T::MODULE, std::mem::size_of::<T::Layout>())
    } {
        Ok(type_object) => type_object,
        Err(e) => type_object_creation_failed(py, e, T::NAME),
    }
}

#[derive(Default)]
struct PyTypeBuilder {
    slots: Vec<ffi::PyType_Slot>,
    method_defs: Vec<ffi::PyMethodDef>,
    property_defs_map: HashMap<&'static str, ffi::PyGetSetDef>,
    /// Used to patch the type objects for the things there's no
    /// PyType_FromSpec API for... there's no reason this should work,
    /// except for that it does and we have tests.
    cleanup: Vec<Box<dyn Fn(&PyTypeBuilder, *mut ffi::PyTypeObject)>>,
    is_mapping: bool,
    has_new: bool,
    has_dealloc: bool,
    has_getitem: bool,
    has_setitem: bool,
    has_traverse: bool,
    has_clear: bool,
    has_dict: bool,
    class_flags: c_ulong,
    // Before Python 3.9, need to patch in buffer methods manually (they don't work in slots)
    #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
    buffer_procs: ffi::PyBufferProcs,
}

impl PyTypeBuilder {
    /// # Safety
    /// The given pointer must be of the correct type for the given slot
    unsafe fn push_slot<T>(&mut self, slot: c_int, pfunc: *mut T) {
        match slot {
            ffi::Py_tp_new => self.has_new = true,
            ffi::Py_tp_dealloc => self.has_dealloc = true,
            ffi::Py_mp_subscript => self.has_getitem = true,
            ffi::Py_mp_ass_subscript => self.has_setitem = true,
            ffi::Py_tp_traverse => {
                self.has_traverse = true;
                self.class_flags |= ffi::Py_TPFLAGS_HAVE_GC;
            }
            ffi::Py_tp_clear => self.has_clear = true,
            #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
            ffi::Py_bf_getbuffer => {
                // Safety: slot.pfunc is a valid function pointer
                self.buffer_procs.bf_getbuffer = Some(std::mem::transmute(pfunc));
            }
            #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
            ffi::Py_bf_releasebuffer => {
                // Safety: slot.pfunc is a valid function pointer
                self.buffer_procs.bf_releasebuffer = Some(std::mem::transmute(pfunc));
            }
            _ => {}
        }

        self.slots.push(ffi::PyType_Slot {
            slot,
            pfunc: pfunc as _,
        });
    }

    /// # Safety
    /// It is the caller's responsibility that `data` is of the correct type for the given slot.
    unsafe fn push_raw_vec<T>(&mut self, slot: c_int, mut data: Vec<T>) {
        if !data.is_empty() {
            // Python expects a zeroed entry to mark the end of the defs
            data.push(std::mem::zeroed());
            self.push_slot(slot, Box::into_raw(data.into_boxed_slice()) as *mut c_void);
        }
    }

    /// # Safety
    /// The given pointer must be of the correct type for the given slot
    unsafe fn with_slot<T>(mut self, slot: c_int, pfunc: *mut T) -> Self {
        self.push_slot(slot, pfunc);
        self
    }

    fn add_pymethod_def(&mut self, def: &PyMethodDefType) {
        const PY_GET_SET_DEF_INIT: ffi::PyGetSetDef = ffi::PyGetSetDef {
            name: ptr::null_mut(),
            get: None,
            set: None,
            doc: ptr::null(),
            closure: ptr::null_mut(),
        };

        match def {
            PyMethodDefType::Getter(getter) => {
                getter.copy_to(
                    self.property_defs_map
                        .entry(getter.name)
                        .or_insert(PY_GET_SET_DEF_INIT),
                );
            }
            PyMethodDefType::Setter(setter) => {
                setter.copy_to(
                    self.property_defs_map
                        .entry(setter.name)
                        .or_insert(PY_GET_SET_DEF_INIT),
                );
            }
            PyMethodDefType::Method(def)
            | PyMethodDefType::Class(def)
            | PyMethodDefType::Static(def) => self.method_defs.push(def.as_method_def().unwrap()),
            // These class attributes are added after the type gets created by LazyStaticType
            PyMethodDefType::ClassAttribute(_) => {}
        }
    }

    fn finalize_methods_and_properties(&mut self) {
        let method_defs = std::mem::take(&mut self.method_defs);
        // Safety: Py_tp_methods expects a raw vec of PyMethodDef
        unsafe { self.push_raw_vec(ffi::Py_tp_methods, method_defs) };

        let property_defs = std::mem::take(&mut self.property_defs_map);
        // TODO: use into_values when on MSRV Rust >= 1.54
        #[allow(unused_mut)]
        let mut property_defs: Vec<_> = property_defs.into_iter().map(|(_, value)| value).collect();

        // PyPy doesn't automatically add __dict__ getter / setter.
        // PyObject_GenericGetDict not in the limited API until Python 3.10.
        if self.has_dict {
            #[cfg(not(any(PyPy, all(Py_LIMITED_API, not(Py_3_10)))))]
            property_defs.push(ffi::PyGetSetDef {
                name: "__dict__\0".as_ptr() as *mut c_char,
                get: Some(ffi::PyObject_GenericGetDict),
                set: Some(ffi::PyObject_GenericSetDict),
                doc: ptr::null(),
                closure: ptr::null_mut(),
            });
        }

        // Safety: Py_tp_members expects a raw vec of PyGetSetDef
        unsafe { self.push_raw_vec(ffi::Py_tp_getset, property_defs) };

        // If mapping methods implemented, define sequence methods get implemented too.
        // CPython does the same for Python `class` statements.

        // NB we don't implement sq_length to avoid annoying CPython behaviour of automatically adding
        // the length to negative indices.

        // Don't add these methods for "pure" mappings.

        if !self.is_mapping && self.has_getitem {
            // Safety: This is the correct slot type for Py_sq_item
            unsafe {
                self.push_slot(
                    ffi::Py_sq_item,
                    get_sequence_item_from_mapping as *mut c_void,
                )
            }
        }

        if !self.is_mapping && self.has_setitem {
            // Safety: This is the correct slot type for Py_sq_ass_item
            unsafe {
                self.push_slot(
                    ffi::Py_sq_ass_item,
                    assign_sequence_item_from_mapping as *mut c_void,
                )
            }
        }
    }

    fn set_is_basetype(mut self, is_basetype: bool) -> Self {
        if is_basetype {
            self.class_flags |= ffi::Py_TPFLAGS_BASETYPE;
        }
        self
    }

    fn set_is_mapping(mut self, is_mapping: bool) -> Self {
        self.is_mapping = is_mapping;
        self
    }

    /// # Safety
    /// All slots in the PyClassItemsIter should be correct
    unsafe fn with_class_items(mut self, iter: PyClassItemsIter) -> Self {
        for items in iter {
            for slot in items.slots {
                self.push_slot(slot.slot, slot.pfunc);
            }
            for method in items.methods {
                self.add_pymethod_def(method);
            }
        }
        self
    }

    fn with_type_doc(mut self, type_doc: &'static str) -> Self {
        if let Some(doc) = py_class_doc(type_doc) {
            unsafe { self.push_slot(ffi::Py_tp_doc, doc) }
        }

        // Running this causes PyPy to segfault.
        #[cfg(all(not(PyPy), not(Py_LIMITED_API), not(Py_3_10)))]
        if type_doc != "\0" {
            // Until CPython 3.10, tp_doc was treated specially for
            // heap-types, and it removed the text_signature value from it.
            // We go in after the fact and replace tp_doc with something
            // that _does_ include the text_signature value!
            self.cleanup
                .push(Box::new(move |_self, type_object| unsafe {
                    ffi::PyObject_Free((*type_object).tp_doc as _);
                    let data = ffi::PyObject_Malloc(type_doc.len());
                    data.copy_from(type_doc.as_ptr() as _, type_doc.len());
                    (*type_object).tp_doc = data as _;
                }))
        }
        self
    }

    fn with_offsets(
        mut self,
        dict_offset: Option<ffi::Py_ssize_t>,
        #[allow(unused_variables)] weaklist_offset: Option<ffi::Py_ssize_t>,
    ) -> Self {
        self.has_dict = dict_offset.is_some();

        #[cfg(Py_3_9)]
        {
            #[inline(always)]
            fn offset_def(
                name: &'static str,
                offset: ffi::Py_ssize_t,
            ) -> ffi::structmember::PyMemberDef {
                ffi::structmember::PyMemberDef {
                    name: name.as_ptr() as _,
                    type_code: ffi::structmember::T_PYSSIZET,
                    offset,
                    flags: ffi::structmember::READONLY,
                    doc: std::ptr::null_mut(),
                }
            }

            let mut members = Vec::new();

            // __dict__ support
            if let Some(dict_offset) = dict_offset {
                members.push(offset_def("__dictoffset__\0", dict_offset));
            }

            // weakref support
            if let Some(weaklist_offset) = weaklist_offset {
                members.push(offset_def("__weaklistoffset__\0", weaklist_offset));
            }

            // Safety: Py_tp_members expects a raw vec of PyMemberDef
            unsafe { self.push_raw_vec(ffi::Py_tp_members, members) };
        }

        // Setting buffer protocols, tp_dictoffset and tp_weaklistoffset via slots doesn't work until
        // Python 3.9, so on older versions we must manually fixup the type object.
        #[cfg(all(not(Py_LIMITED_API), not(Py_3_9)))]
        {
            self.cleanup
                .push(Box::new(move |builder, type_object| unsafe {
                    (*(*type_object).tp_as_buffer).bf_getbuffer = builder.buffer_procs.bf_getbuffer;
                    (*(*type_object).tp_as_buffer).bf_releasebuffer =
                        builder.buffer_procs.bf_releasebuffer;

                    if let Some(dict_offset) = dict_offset {
                        (*type_object).tp_dictoffset = dict_offset;
                    }

                    if let Some(weaklist_offset) = weaklist_offset {
                        (*type_object).tp_weaklistoffset = weaklist_offset;
                    }
                }));
        }
        self
    }

    fn build(
        mut self,
        py: Python<'_>,
        name: &'static str,
        module_name: Option<&'static str>,
        basicsize: usize,
    ) -> PyResult<*mut ffi::PyTypeObject> {
        self.finalize_methods_and_properties();

        if !self.has_new {
            // Safety: This is the correct slot type for Py_tp_new
            unsafe { self.push_slot(ffi::Py_tp_new, no_constructor_defined as *mut c_void) }
        }

        if !self.has_dealloc {
            panic!("PyTypeBuilder requires you to specify slot ffi::Py_tp_dealloc");
        }

        if self.has_clear && !self.has_traverse {
            return Err(PyTypeError::new_err(format!(
                "`#[pyclass]` {} implements __clear__ without __traverse__",
                name
            )));
        }

        // Add empty sentinel at the end
        // Safety: python expects this empty slot
        unsafe { self.push_slot(0, ptr::null_mut::<c_void>()) }

        let mut spec = ffi::PyType_Spec {
            name: py_class_qualified_name(module_name, name)?,
            basicsize: basicsize as c_int,
            itemsize: 0,
            // `c_ulong` and `c_uint` have the same size
            // on some platforms (like windows)
            #[allow(clippy::useless_conversion)]
            flags: (ffi::Py_TPFLAGS_DEFAULT | self.class_flags)
                .try_into()
                .unwrap(),
            slots: self.slots.as_mut_ptr(),
        };

        // Safety: We've correctly setup the PyType_Spec at this point
        let type_object = unsafe { ffi::PyType_FromSpec(&mut spec) };
        if type_object.is_null() {
            Err(PyErr::fetch(py))
        } else {
            for cleanup in std::mem::take(&mut self.cleanup) {
                cleanup(&self, type_object as _);
            }

            Ok(type_object as _)
        }
    }
}

#[cold]
fn type_object_creation_failed(py: Python<'_>, e: PyErr, name: &str) -> ! {
    e.print(py);
    panic!("An error occurred while initializing class {}", name)
}

fn py_class_doc(class_doc: &str) -> Option<*mut c_char> {
    match class_doc {
        "\0" => None,
        s => {
            // To pass *mut pointer to python safely, leak a CString in whichever case
            let cstring = if s.as_bytes().last() == Some(&0) {
                CStr::from_bytes_with_nul(s.as_bytes())
                    .unwrap_or_else(|e| panic!("doc contains interior nul byte: {:?} in {}", e, s))
                    .to_owned()
            } else {
                CString::new(s)
                    .unwrap_or_else(|e| panic!("doc contains interior nul byte: {:?} in {}", e, s))
            };
            Some(cstring.into_raw())
        }
    }
}

fn py_class_qualified_name(module_name: Option<&str>, class_name: &str) -> PyResult<*mut c_char> {
    Ok(CString::new(format!(
        "{}.{}",
        module_name.unwrap_or("builtins"),
        class_name
    ))?
    .into_raw())
}

/// Operators for the `__richcmp__` method
#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    /// The *less than* operator.
    Lt = ffi::Py_LT as isize,
    /// The *less than or equal to* operator.
    Le = ffi::Py_LE as isize,
    /// The equality operator.
    Eq = ffi::Py_EQ as isize,
    /// The *not equal to* operator.
    Ne = ffi::Py_NE as isize,
    /// The *greater than* operator.
    Gt = ffi::Py_GT as isize,
    /// The *greater than or equal to* operator.
    Ge = ffi::Py_GE as isize,
}

impl CompareOp {
    /// Conversion from the C enum.
    pub fn from_raw(op: c_int) -> Option<Self> {
        match op {
            ffi::Py_LT => Some(CompareOp::Lt),
            ffi::Py_LE => Some(CompareOp::Le),
            ffi::Py_EQ => Some(CompareOp::Eq),
            ffi::Py_NE => Some(CompareOp::Ne),
            ffi::Py_GT => Some(CompareOp::Gt),
            ffi::Py_GE => Some(CompareOp::Ge),
            _ => None,
        }
    }

    /// Returns if a Rust [`std::cmp::Ordering`] matches this ordering query.
    ///
    /// Usage example:
    ///
    /// ```rust
    /// # use pyo3::prelude::*;
    /// # use pyo3::class::basic::CompareOp;
    ///
    /// #[pyclass]
    /// struct Size {
    ///     size: usize
    /// }
    ///
    /// #[pymethods]
    /// impl Size {
    ///     fn __richcmp__(&self, other: &Size, op: CompareOp) -> bool {
    ///         op.matches(self.size.cmp(&other.size))
    ///     }
    /// }
    /// ```
    pub fn matches(&self, result: Ordering) -> bool {
        match self {
            CompareOp::Eq => result == Ordering::Equal,
            CompareOp::Ne => result != Ordering::Equal,
            CompareOp::Lt => result == Ordering::Less,
            CompareOp::Le => result != Ordering::Greater,
            CompareOp::Gt => result == Ordering::Greater,
            CompareOp::Ge => result != Ordering::Less,
        }
    }
}

/// Output of `__next__` which can either `yield` the next value in the iteration, or
/// `return` a value to raise `StopIteration` in Python.
///
/// See [`PyIterProtocol`](trait.PyIterProtocol.html) for an example.
pub enum IterNextOutput<T, U> {
    /// The value yielded by the iterator.
    Yield(T),
    /// The `StopIteration` object.
    Return(U),
}

pub type PyIterNextOutput = IterNextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterNextOutput {
    fn convert(self, _py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterNextOutput::Yield(o) => Ok(o.into_ptr()),
            IterNextOutput::Return(opt) => Err(crate::exceptions::PyStopIteration::new_err((opt,))),
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterNextOutput> for IterNextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterNextOutput> {
        match self {
            IterNextOutput::Yield(o) => Ok(IterNextOutput::Yield(o.into_py(py))),
            IterNextOutput::Return(o) => Ok(IterNextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterNextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterNextOutput> {
        match self {
            Some(o) => Ok(PyIterNextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterNextOutput::Return(py.None())),
        }
    }
}

/// Output of `__anext__`.
///
/// <https://docs.python.org/3/reference/expressions.html#agen.__anext__>
pub enum IterANextOutput<T, U> {
    /// An expression which the generator yielded.
    Yield(T),
    /// A `StopAsyncIteration` object.
    Return(U),
}

/// An [IterANextOutput] of Python objects.
pub type PyIterANextOutput = IterANextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterANextOutput {
    fn convert(self, _py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterANextOutput::Yield(o) => Ok(o.into_ptr()),
            IterANextOutput::Return(opt) => {
                Err(crate::exceptions::PyStopAsyncIteration::new_err((opt,)))
            }
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterANextOutput> for IterANextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterANextOutput> {
        match self {
            IterANextOutput::Yield(o) => Ok(IterANextOutput::Yield(o.into_py(py))),
            IterANextOutput::Return(o) => Ok(IterANextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterANextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python<'_>) -> PyResult<PyIterANextOutput> {
        match self {
            Some(o) => Ok(PyIterANextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterANextOutput::Return(py.None())),
        }
    }
}

/// Default new implementation
pub(crate) unsafe extern "C" fn no_constructor_defined(
    _subtype: *mut ffi::PyTypeObject,
    _args: *mut ffi::PyObject,
    _kwds: *mut ffi::PyObject,
) -> *mut ffi::PyObject {
    crate::callback_body!(py, {
        Err::<(), _>(crate::exceptions::PyTypeError::new_err(
            "No constructor defined",
        ))
    })
}

/// A workaround for [associated const equality](https://github.com/rust-lang/rust/issues/92827).
///
/// This serves to have True / False values in the [`PyClass`] trait's `Frozen` type.
#[doc(hidden)]
pub mod boolean_struct {
    pub(crate) mod private {
        use super::*;

        /// A way to "seal" the boolean traits.
        pub trait Boolean {}

        impl Boolean for True {}
        impl Boolean for False {}
    }

    pub struct True(());
    pub struct False(());
}

/// A trait which is used to describe whether a `#[pyclass]` is frozen.
#[doc(hidden)]
pub trait Frozen: boolean_struct::private::Boolean {}

impl Frozen for boolean_struct::True {}
impl Frozen for boolean_struct::False {}

mod tests {
    #[test]
    fn test_compare_op_matches() {
        use super::CompareOp;
        use std::cmp::Ordering;

        assert!(CompareOp::Eq.matches(Ordering::Equal));
        assert!(CompareOp::Ne.matches(Ordering::Less));
        assert!(CompareOp::Ge.matches(Ordering::Greater));
        assert!(CompareOp::Gt.matches(Ordering::Greater));
        assert!(CompareOp::Le.matches(Ordering::Equal));
        assert!(CompareOp::Lt.matches(Ordering::Less));
    }
}
