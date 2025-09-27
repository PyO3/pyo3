use crate::{
    exceptions::PyTypeError,
    ffi,
    impl_::{
        pycell::PyClassObject,
        pyclass::{
            assign_sequence_item_from_mapping, get_sequence_item_from_mapping, tp_dealloc,
            tp_dealloc_with_gc, MaybeRuntimePyMethodDef, PyClassItemsIter,
        },
        pymethods::{Getter, PyGetterDef, PyMethodDefType, PySetterDef, Setter, _call_clear},
        trampoline::trampoline,
    },
    internal_tricks::ptr_from_ref,
    types::{typeobject::PyTypeMethods, PyType},
    Py, PyClass, PyResult, PyTypeInfo, Python,
};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_ulong, c_void},
    ptr,
};

pub(crate) struct PyClassTypeObject {
    pub type_object: Py<PyType>,
    pub is_immutable_type: bool,
    #[allow(dead_code)] // This is purely a cache that must live as long as the type object
    getset_destructors: Vec<GetSetDefDestructor>,
}

pub(crate) fn create_type_object<T>(py: Python<'_>) -> PyResult<PyClassTypeObject>
where
    T: PyClass,
{
    // Written this way to monomorphize the majority of the logic.
    #[allow(clippy::too_many_arguments)]
    unsafe fn inner(
        py: Python<'_>,
        base: *mut ffi::PyTypeObject,
        dealloc: unsafe extern "C" fn(*mut ffi::PyObject),
        dealloc_with_gc: unsafe extern "C" fn(*mut ffi::PyObject),
        is_mapping: bool,
        is_sequence: bool,
        is_immutable_type: bool,
        doc: &'static CStr,
        dict_offset: Option<ffi::Py_ssize_t>,
        weaklist_offset: Option<ffi::Py_ssize_t>,
        is_basetype: bool,
        items_iter: PyClassItemsIter,
        name: &'static str,
        module: Option<&'static str>,
        size_of: usize,
    ) -> PyResult<PyClassTypeObject> {
        unsafe {
            PyTypeBuilder {
                slots: Vec::new(),
                method_defs: Vec::new(),
                member_defs: Vec::new(),
                getset_builders: HashMap::new(),
                cleanup: Vec::new(),
                tp_base: base,
                tp_dealloc: dealloc,
                tp_dealloc_with_gc: dealloc_with_gc,
                is_mapping,
                is_sequence,
                is_immutable_type,
                has_new: false,
                has_dealloc: false,
                has_getitem: false,
                has_setitem: false,
                has_traverse: false,
                has_clear: false,
                dict_offset: None,
                class_flags: 0,
                #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
                buffer_procs: Default::default(),
            }
            .type_doc(doc)
            .offsets(dict_offset, weaklist_offset)
            .set_is_basetype(is_basetype)
            .class_items(items_iter)
            .build(py, name, module, size_of)
        }
    }

    unsafe {
        inner(
            py,
            T::BaseType::type_object_raw(py),
            tp_dealloc::<T>,
            tp_dealloc_with_gc::<T>,
            T::IS_MAPPING,
            T::IS_SEQUENCE,
            T::IS_IMMUTABLE_TYPE,
            T::DOC,
            T::dict_offset(),
            T::weaklist_offset(),
            T::IS_BASETYPE,
            T::items_iter(),
            T::NAME,
            T::MODULE,
            std::mem::size_of::<PyClassObject<T>>(),
        )
    }
}

type PyTypeBuilderCleanup = Box<dyn Fn(&PyTypeBuilder, *mut ffi::PyTypeObject)>;

struct PyTypeBuilder {
    slots: Vec<ffi::PyType_Slot>,
    method_defs: Vec<ffi::PyMethodDef>,
    member_defs: Vec<ffi::PyMemberDef>,
    getset_builders: HashMap<&'static CStr, GetSetDefBuilder>,
    /// Used to patch the type objects for the things there's no
    /// PyType_FromSpec API for... there's no reason this should work,
    /// except for that it does and we have tests.
    cleanup: Vec<PyTypeBuilderCleanup>,
    tp_base: *mut ffi::PyTypeObject,
    tp_dealloc: ffi::destructor,
    tp_dealloc_with_gc: ffi::destructor,
    is_mapping: bool,
    is_sequence: bool,
    is_immutable_type: bool,
    has_new: bool,
    has_dealloc: bool,
    has_getitem: bool,
    has_setitem: bool,
    has_traverse: bool,
    has_clear: bool,
    dict_offset: Option<ffi::Py_ssize_t>,
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
                self.buffer_procs.bf_getbuffer =
                    Some(unsafe { std::mem::transmute::<*mut T, ffi::getbufferproc>(pfunc) });
            }
            #[cfg(all(not(Py_3_9), not(Py_LIMITED_API)))]
            ffi::Py_bf_releasebuffer => {
                // Safety: slot.pfunc is a valid function pointer
                self.buffer_procs.bf_releasebuffer =
                    Some(unsafe { std::mem::transmute::<*mut T, ffi::releasebufferproc>(pfunc) });
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
    unsafe fn push_raw_vec_slot<T>(&mut self, slot: c_int, mut data: Vec<T>) {
        if !data.is_empty() {
            // Python expects a zeroed entry to mark the end of the defs
            unsafe {
                data.push(std::mem::zeroed());
                self.push_slot(slot, Box::into_raw(data.into_boxed_slice()) as *mut c_void);
            }
        }
    }

    fn pymethod_def(&mut self, def: &PyMethodDefType) {
        match def {
            PyMethodDefType::Getter(getter) => self
                .getset_builders
                .entry(getter.name)
                .or_default()
                .add_getter(getter),
            PyMethodDefType::Setter(setter) => self
                .getset_builders
                .entry(setter.name)
                .or_default()
                .add_setter(setter),
            PyMethodDefType::Method(def)
            | PyMethodDefType::Class(def)
            | PyMethodDefType::Static(def) => self.method_defs.push(def.as_method_def()),
            // These class attributes are added after the type gets created by LazyStaticType
            PyMethodDefType::ClassAttribute(_) => {}
            PyMethodDefType::StructMember(def) => self.member_defs.push(*def),
        }
    }

    fn finalize_methods_and_properties(&mut self) -> Vec<GetSetDefDestructor> {
        let method_defs: Vec<pyo3_ffi::PyMethodDef> = std::mem::take(&mut self.method_defs);
        // Safety: Py_tp_methods expects a raw vec of PyMethodDef
        unsafe { self.push_raw_vec_slot(ffi::Py_tp_methods, method_defs) };

        let member_defs = std::mem::take(&mut self.member_defs);
        // Safety: Py_tp_members expects a raw vec of PyMemberDef
        unsafe { self.push_raw_vec_slot(ffi::Py_tp_members, member_defs) };

        let mut getset_destructors = Vec::with_capacity(self.getset_builders.len());

        #[allow(unused_mut)]
        let mut property_defs: Vec<_> = self
            .getset_builders
            .iter()
            .map(|(name, builder)| {
                let (def, destructor) = builder.as_get_set_def(name);
                getset_destructors.push(destructor);
                def
            })
            .collect();

        // PyPy automatically adds __dict__ getter / setter.
        #[cfg(not(PyPy))]
        // Supported on unlimited API for all versions, and on 3.9+ for limited API
        #[cfg(any(Py_3_9, not(Py_LIMITED_API)))]
        if let Some(dict_offset) = self.dict_offset {
            let get_dict;
            let closure;
            // PyObject_GenericGetDict not in the limited API until Python 3.10.
            #[cfg(any(not(Py_LIMITED_API), Py_3_10))]
            {
                let _ = dict_offset;
                get_dict = ffi::PyObject_GenericGetDict;
                closure = ptr::null_mut();
            }

            // ... so we write a basic implementation ourselves
            #[cfg(not(any(not(Py_LIMITED_API), Py_3_10)))]
            {
                extern "C" fn get_dict_impl(
                    object: *mut ffi::PyObject,
                    closure: *mut c_void,
                ) -> *mut ffi::PyObject {
                    unsafe {
                        trampoline(|_| {
                            let dict_offset = closure as ffi::Py_ssize_t;
                            // we don't support negative dict_offset here; PyO3 doesn't set it negative
                            assert!(dict_offset > 0);
                            // TODO: use `.byte_offset` on MSRV 1.75
                            let dict_ptr = object
                                .cast::<u8>()
                                .offset(dict_offset)
                                .cast::<*mut ffi::PyObject>();
                            if (*dict_ptr).is_null() {
                                std::ptr::write(dict_ptr, ffi::PyDict_New());
                            }
                            Ok(ffi::compat::Py_XNewRef(*dict_ptr))
                        })
                    }
                }

                get_dict = get_dict_impl;
                closure = dict_offset as _;
            }

            property_defs.push(ffi::PyGetSetDef {
                name: ffi::c_str!("__dict__").as_ptr(),
                get: Some(get_dict),
                set: Some(ffi::PyObject_GenericSetDict),
                doc: ptr::null(),
                closure,
            });
        }

        // Safety: Py_tp_getset expects a raw vec of PyGetSetDef
        unsafe { self.push_raw_vec_slot(ffi::Py_tp_getset, property_defs) };

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

        getset_destructors
    }

    fn set_is_basetype(mut self, is_basetype: bool) -> Self {
        if is_basetype {
            self.class_flags |= ffi::Py_TPFLAGS_BASETYPE;
        }
        self
    }

    /// # Safety
    /// All slots in the PyClassItemsIter should be correct
    unsafe fn class_items(mut self, iter: PyClassItemsIter) -> Self {
        for items in iter {
            for slot in items.slots {
                unsafe { self.push_slot(slot.slot, slot.pfunc) };
            }
            for method in items.methods {
                let built_method;
                let method = match method {
                    MaybeRuntimePyMethodDef::Runtime(builder) => {
                        built_method = builder();
                        &built_method
                    }
                    MaybeRuntimePyMethodDef::Static(method) => method,
                };
                self.pymethod_def(method);
            }
        }
        self
    }

    fn type_doc(mut self, type_doc: &'static CStr) -> Self {
        let slice = type_doc.to_bytes();
        if !slice.is_empty() {
            unsafe { self.push_slot(ffi::Py_tp_doc, type_doc.as_ptr() as *mut c_char) }

            // Running this causes PyPy to segfault.
            #[cfg(all(not(PyPy), not(Py_LIMITED_API), not(Py_3_10)))]
            {
                // Until CPython 3.10, tp_doc was treated specially for
                // heap-types, and it removed the text_signature value from it.
                // We go in after the fact and replace tp_doc with something
                // that _does_ include the text_signature value!
                self.cleanup
                    .push(Box::new(move |_self, type_object| unsafe {
                        ffi::PyObject_Free((*type_object).tp_doc as _);
                        let data = ffi::PyMem_Malloc(slice.len());
                        data.copy_from(slice.as_ptr() as _, slice.len());
                        (*type_object).tp_doc = data as _;
                    }))
            }
        }
        self
    }

    fn offsets(
        mut self,
        dict_offset: Option<ffi::Py_ssize_t>,
        #[allow(unused_variables)] weaklist_offset: Option<ffi::Py_ssize_t>,
    ) -> Self {
        self.dict_offset = dict_offset;

        #[cfg(Py_3_9)]
        {
            #[inline(always)]
            fn offset_def(name: &'static CStr, offset: ffi::Py_ssize_t) -> ffi::PyMemberDef {
                ffi::PyMemberDef {
                    name: name.as_ptr().cast(),
                    type_code: ffi::Py_T_PYSSIZET,
                    offset,
                    flags: ffi::Py_READONLY,
                    doc: std::ptr::null_mut(),
                }
            }

            // __dict__ support
            if let Some(dict_offset) = dict_offset {
                self.member_defs
                    .push(offset_def(ffi::c_str!("__dictoffset__"), dict_offset));
            }

            // weakref support
            if let Some(weaklist_offset) = weaklist_offset {
                self.member_defs.push(offset_def(
                    ffi::c_str!("__weaklistoffset__"),
                    weaklist_offset,
                ));
            }
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
    ) -> PyResult<PyClassTypeObject> {
        // `c_ulong` and `c_uint` have the same size
        // on some platforms (like windows)
        #![allow(clippy::useless_conversion)]

        let getset_destructors = self.finalize_methods_and_properties();

        unsafe { self.push_slot(ffi::Py_tp_base, self.tp_base) }

        if !self.has_new {
            // Flag introduced in 3.10, only worked in PyPy on 3.11
            #[cfg(not(any(all(Py_3_10, not(PyPy)), all(Py_3_11, PyPy))))]
            {
                // Safety: This is the correct slot type for Py_tp_new
                unsafe { self.push_slot(ffi::Py_tp_new, no_constructor_defined as *mut c_void) }
            }
            #[cfg(any(all(Py_3_10, not(PyPy)), all(Py_3_11, PyPy)))]
            {
                self.class_flags |= ffi::Py_TPFLAGS_DISALLOW_INSTANTIATION;
            }
        }

        let base_is_gc = unsafe { ffi::PyType_IS_GC(self.tp_base) == 1 };
        let tp_dealloc = if self.has_traverse || base_is_gc {
            self.tp_dealloc_with_gc
        } else {
            self.tp_dealloc
        };
        unsafe { self.push_slot(ffi::Py_tp_dealloc, tp_dealloc as *mut c_void) }

        if self.has_clear && !self.has_traverse {
            return Err(PyTypeError::new_err(format!(
                "`#[pyclass]` {name} implements __clear__ without __traverse__"
            )));
        }

        // If this type is a GC type, and the base also is, we may need to add
        // `tp_traverse` / `tp_clear` implementations to call the base, if this type didn't
        // define `__traverse__` or `__clear__`.
        //
        // This is because when Py_TPFLAGS_HAVE_GC is set, then `tp_traverse` and
        // `tp_clear` are not inherited.
        if ((self.class_flags & ffi::Py_TPFLAGS_HAVE_GC) != 0) && base_is_gc {
            // If this assertion breaks, need to consider doing the same for __traverse__.
            assert!(self.has_traverse); // Py_TPFLAGS_HAVE_GC is set when a `__traverse__` method is found

            if !self.has_clear {
                // Safety: This is the correct slot type for Py_tp_clear
                unsafe { self.push_slot(ffi::Py_tp_clear, call_super_clear as *mut c_void) }
            }
        }

        // For sequences, implement sq_length instead of mp_length
        if self.is_sequence {
            for slot in &mut self.slots {
                if slot.slot == ffi::Py_mp_length {
                    slot.slot = ffi::Py_sq_length;
                }
            }
        }

        // Add empty sentinel at the end
        // Safety: python expects this empty slot
        unsafe { self.push_slot(0, ptr::null_mut::<c_void>()) }

        let class_name = py_class_qualified_name(module_name, name)?;
        let mut spec = ffi::PyType_Spec {
            name: class_name.as_ptr() as _,
            basicsize: basicsize as c_int,
            itemsize: 0,

            flags: (ffi::Py_TPFLAGS_DEFAULT | self.class_flags)
                .try_into()
                .unwrap(),
            slots: self.slots.as_mut_ptr(),
        };

        // Safety: We've correctly setup the PyType_Spec at this point
        let type_object: Py<PyType> =
            unsafe { Py::from_owned_ptr_or_err(py, ffi::PyType_FromSpec(&mut spec))? };

        #[cfg(not(Py_3_11))]
        bpo_45315_workaround(py, class_name);

        for cleanup in std::mem::take(&mut self.cleanup) {
            cleanup(&self, type_object.bind(py).as_type_ptr());
        }

        Ok(PyClassTypeObject {
            type_object,
            is_immutable_type: self.is_immutable_type,
            getset_destructors,
        })
    }
}

fn py_class_qualified_name(module_name: Option<&str>, class_name: &str) -> PyResult<CString> {
    Ok(CString::new(format!(
        "{}.{}",
        module_name.unwrap_or("builtins"),
        class_name
    ))?)
}

/// Workaround for Python issue 45315; no longer necessary in Python 3.11
#[inline]
#[cfg(not(Py_3_11))]
fn bpo_45315_workaround(py: Python<'_>, class_name: CString) {
    #[cfg(Py_LIMITED_API)]
    {
        // Must check version at runtime for abi3 wheels - they could run against a higher version
        // than the build config suggests.
        use crate::sync::PyOnceLock;
        static IS_PYTHON_3_11: PyOnceLock<bool> = PyOnceLock::new();

        if *IS_PYTHON_3_11.get_or_init(py, || py.version_info() >= (3, 11)) {
            // No fix needed - the wheel is running on a sufficiently new interpreter.
            return;
        }
    }
    #[cfg(not(Py_LIMITED_API))]
    {
        // suppress unused variable warning
        let _ = py;
    }

    std::mem::forget(class_name);
}

/// Default new implementation
#[cfg(not(any(all(Py_3_10, not(PyPy)), all(Py_3_11, PyPy))))]
unsafe extern "C" fn no_constructor_defined(
    subtype: *mut ffi::PyTypeObject,
    _args: *mut ffi::PyObject,
    _kwds: *mut ffi::PyObject,
) -> *mut ffi::PyObject {
    unsafe {
        trampoline(|py| {
            let tpobj = PyType::from_borrowed_type_ptr(py, subtype);
            // unlike `fully_qualified_name`, this always include the module
            let module = tpobj
                .module()
                .map_or_else(|_| "<unknown>".into(), |s| s.to_string());
            let qualname = tpobj.qualname();
            let qualname = qualname.map_or_else(|_| "<unknown>".into(), |s| s.to_string());
            Err(crate::exceptions::PyTypeError::new_err(format!(
                "cannot create '{module}.{qualname}' instances"
            )))
        })
    }
}

unsafe extern "C" fn call_super_clear(slf: *mut ffi::PyObject) -> c_int {
    unsafe { _call_clear(slf, |_, _| Ok(()), call_super_clear) }
}

#[derive(Default)]
struct GetSetDefBuilder {
    doc: Option<&'static CStr>,
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl GetSetDefBuilder {
    fn add_getter(&mut self, getter: &PyGetterDef) {
        // TODO: be smarter about merging getter and setter docs
        if self.doc.is_none() {
            self.doc = Some(getter.doc);
        }
        // TODO: return an error if getter already defined?
        self.getter = Some(getter.meth)
    }

    fn add_setter(&mut self, setter: &PySetterDef) {
        // TODO: be smarter about merging getter and setter docs
        if self.doc.is_none() {
            self.doc = Some(setter.doc);
        }
        // TODO: return an error if setter already defined?
        self.setter = Some(setter.meth)
    }

    fn as_get_set_def(&self, name: &'static CStr) -> (ffi::PyGetSetDef, GetSetDefDestructor) {
        let getset_type = match (self.getter, self.setter) {
            (Some(getter), None) => GetSetDefType::Getter(getter),
            (None, Some(setter)) => GetSetDefType::Setter(setter),
            (Some(getter), Some(setter)) => {
                GetSetDefType::GetterAndSetter(Box::new(GetterAndSetter { getter, setter }))
            }
            (None, None) => {
                unreachable!("GetSetDefBuilder expected to always have either getter or setter")
            }
        };

        let getset_def = getset_type.create_py_get_set_def(name, self.doc);
        let destructor = GetSetDefDestructor {
            closure: getset_type,
        };
        (getset_def, destructor)
    }
}

#[allow(dead_code)] // a stack of fields which are purely to cache until dropped
struct GetSetDefDestructor {
    closure: GetSetDefType,
}

/// Possible forms of property - either a getter, setter, or both
enum GetSetDefType {
    Getter(Getter),
    Setter(Setter),
    // The box is here so that the `GetterAndSetter` has a stable
    // memory address even if the `GetSetDefType` enum is moved
    GetterAndSetter(Box<GetterAndSetter>),
}

pub(crate) struct GetterAndSetter {
    getter: Getter,
    setter: Setter,
}

impl GetSetDefType {
    /// Fills a PyGetSetDef structure
    /// It is only valid for as long as this GetSetDefType remains alive,
    /// as well as name and doc members
    pub(crate) fn create_py_get_set_def(
        &self,
        name: &CStr,
        doc: Option<&CStr>,
    ) -> ffi::PyGetSetDef {
        let (get, set, closure): (Option<ffi::getter>, Option<ffi::setter>, *mut c_void) =
            match self {
                &Self::Getter(closure) => {
                    unsafe extern "C" fn getter(
                        slf: *mut ffi::PyObject,
                        closure: *mut c_void,
                    ) -> *mut ffi::PyObject {
                        // Safety: PyO3 sets the closure when constructing the ffi getter so this cast should always be valid
                        let getter: Getter = unsafe { std::mem::transmute(closure) };
                        unsafe { trampoline(|py| getter(py, slf)) }
                    }
                    (Some(getter), None, closure as Getter as _)
                }
                &Self::Setter(closure) => {
                    unsafe extern "C" fn setter(
                        slf: *mut ffi::PyObject,
                        value: *mut ffi::PyObject,
                        closure: *mut c_void,
                    ) -> c_int {
                        // Safety: PyO3 sets the closure when constructing the ffi setter so this cast should always be valid
                        let setter: Setter = unsafe { std::mem::transmute(closure) };
                        unsafe { trampoline(|py| setter(py, slf, value)) }
                    }
                    (None, Some(setter), closure as Setter as _)
                }
                Self::GetterAndSetter(closure) => {
                    unsafe extern "C" fn getset_getter(
                        slf: *mut ffi::PyObject,
                        closure: *mut c_void,
                    ) -> *mut ffi::PyObject {
                        let getset: &GetterAndSetter = unsafe { &*closure.cast() };
                        unsafe { trampoline(|py| (getset.getter)(py, slf)) }
                    }

                    unsafe extern "C" fn getset_setter(
                        slf: *mut ffi::PyObject,
                        value: *mut ffi::PyObject,
                        closure: *mut c_void,
                    ) -> c_int {
                        let getset: &GetterAndSetter = unsafe { &*closure.cast() };
                        unsafe { trampoline(|py| (getset.setter)(py, slf, value)) }
                    }
                    (
                        Some(getset_getter),
                        Some(getset_setter),
                        ptr_from_ref::<GetterAndSetter>(closure) as *mut _,
                    )
                }
            };
        ffi::PyGetSetDef {
            name: name.as_ptr(),
            doc: doc.map_or(ptr::null(), CStr::as_ptr),
            get,
            set,
            closure,
        }
    }
}
