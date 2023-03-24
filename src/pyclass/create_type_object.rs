use crate::{
    exceptions::PyTypeError,
    ffi,
    impl_::pyclass::{
        assign_sequence_item_from_mapping, get_sequence_item_from_mapping, tp_dealloc,
        PyClassItemsIter,
    },
    types::PyType,
    Py, PyClass, PyMethodDefType, PyResult, PyTypeInfo, Python,
};
use std::{
    collections::HashMap,
    convert::TryInto,
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_ulong, c_void},
    ptr,
};

pub(crate) fn create_type_object<T>(py: Python<'_>) -> PyResult<Py<PyType>>
where
    T: PyClass,
{
    unsafe {
        PyTypeBuilder::default()
            .type_doc(T::DOC)
            .offsets(T::dict_offset(), T::weaklist_offset())
            .slot(ffi::Py_tp_base, T::BaseType::type_object_raw(py))
            .slot(ffi::Py_tp_dealloc, tp_dealloc::<T> as *mut c_void)
            .set_is_basetype(T::IS_BASETYPE)
            .set_is_mapping(T::IS_MAPPING)
            .set_is_sequence(T::IS_SEQUENCE)
            .class_items(T::items_iter())
            .build(py, T::NAME, T::MODULE, std::mem::size_of::<T::Layout>())
    }
}

type PyTypeBuilderCleanup = Box<dyn Fn(&PyTypeBuilder, *mut ffi::PyTypeObject)>;

#[derive(Default)]
struct PyTypeBuilder {
    slots: Vec<ffi::PyType_Slot>,
    method_defs: Vec<ffi::PyMethodDef>,
    property_defs_map: HashMap<&'static str, ffi::PyGetSetDef>,
    /// Used to patch the type objects for the things there's no
    /// PyType_FromSpec API for... there's no reason this should work,
    /// except for that it does and we have tests.
    cleanup: Vec<PyTypeBuilderCleanup>,
    is_mapping: bool,
    is_sequence: bool,
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
    unsafe fn push_raw_vec_slot<T>(&mut self, slot: c_int, mut data: Vec<T>) {
        if !data.is_empty() {
            // Python expects a zeroed entry to mark the end of the defs
            data.push(std::mem::zeroed());
            self.push_slot(slot, Box::into_raw(data.into_boxed_slice()) as *mut c_void);
        }
    }

    /// # Safety
    /// The given pointer must be of the correct type for the given slot
    unsafe fn slot<T>(mut self, slot: c_int, pfunc: *mut T) -> Self {
        self.push_slot(slot, pfunc);
        self
    }

    fn pymethod_def(&mut self, def: &PyMethodDefType) {
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
            | PyMethodDefType::Static(def) => {
                let (def, destructor) = def.as_method_def().unwrap();
                // FIXME: stop leaking destructor
                std::mem::forget(destructor);
                self.method_defs.push(def);
            }
            // These class attributes are added after the type gets created by LazyStaticType
            PyMethodDefType::ClassAttribute(_) => {}
        }
    }

    fn finalize_methods_and_properties(&mut self) {
        let method_defs = std::mem::take(&mut self.method_defs);
        // Safety: Py_tp_methods expects a raw vec of PyMethodDef
        unsafe { self.push_raw_vec_slot(ffi::Py_tp_methods, method_defs) };

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

    fn set_is_sequence(mut self, is_sequence: bool) -> Self {
        self.is_sequence = is_sequence;
        self
    }

    /// # Safety
    /// All slots in the PyClassItemsIter should be correct
    unsafe fn class_items(mut self, iter: PyClassItemsIter) -> Self {
        for items in iter {
            for slot in items.slots {
                self.push_slot(slot.slot, slot.pfunc);
            }
            for method in items.methods {
                self.pymethod_def(method);
            }
        }
        self
    }

    fn type_doc(mut self, type_doc: &'static str) -> Self {
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

    fn offsets(
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
            unsafe { self.push_raw_vec_slot(ffi::Py_tp_members, members) };
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
    ) -> PyResult<Py<PyType>> {
        // `c_ulong` and `c_uint` have the same size
        // on some platforms (like windows)
        #![allow(clippy::useless_conversion)]

        self.finalize_methods_and_properties();

        if !self.has_new {
            // Safety: This is the correct slot type for Py_tp_new
            unsafe { self.push_slot(ffi::Py_tp_new, no_constructor_defined as *mut c_void) }
        }

        assert!(
            self.has_dealloc,
            "PyTypeBuilder requires you to specify slot ffi::Py_tp_dealloc"
        );

        if self.has_clear && !self.has_traverse {
            return Err(PyTypeError::new_err(format!(
                "`#[pyclass]` {} implements __clear__ without __traverse__",
                name
            )));
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

        let mut spec = ffi::PyType_Spec {
            name: py_class_qualified_name(module_name, name)?,
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

        for cleanup in std::mem::take(&mut self.cleanup) {
            cleanup(&self, type_object.as_ref(py).as_type_ptr());
        }

        Ok(type_object)
    }
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

/// Default new implementation
unsafe extern "C" fn no_constructor_defined(
    _subtype: *mut ffi::PyTypeObject,
    _args: *mut ffi::PyObject,
    _kwds: *mut ffi::PyObject,
) -> *mut ffi::PyObject {
    crate::impl_::trampoline::trampoline_inner(|_| {
        Err(crate::exceptions::PyTypeError::new_err(
            "No constructor defined",
        ))
    })
}
