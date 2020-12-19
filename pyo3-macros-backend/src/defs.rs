// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::proto_method::MethodProto;
use std::collections::HashSet;

/// Predicates for `#[pyproto]`.
pub struct Proto {
    /// The name of this protocol. E.g., Iter.
    pub name: &'static str,
    /// Trait which stores the slots
    pub slots_trait: &'static str,
    /// Trait method which accesses the slots.
    pub slots_trait_slots: &'static str,
    /// All methods.
    pub methods: &'static [MethodProto],
    /// All methods registered as normal methods like `#[pymethods]`.
    pub py_methods: &'static [PyMethod],
    /// All methods registered to the slot table.
    slot_defs: &'static [SlotDef],
}

impl Proto {
    pub(crate) fn get_proto<Q>(&self, query: Q) -> Option<&'static MethodProto>
    where
        Q: PartialEq<&'static str>,
    {
        self.methods.iter().find(|m| query == m.name)
    }
    pub(crate) fn get_method<Q>(&self, query: Q) -> Option<&'static PyMethod>
    where
        Q: PartialEq<&'static str>,
    {
        self.py_methods.iter().find(|m| query == m.name)
    }
    // Since the order matters, we expose only the iterator instead of the slice.
    pub(crate) fn slot_defs(
        &self,
        mut implemented_protocols: HashSet<String>,
    ) -> impl Iterator<Item = &'static SlotDef> {
        self.slot_defs.iter().filter(move |slot_def| {
            // If any required method is not implemented, we skip this def.
            let all_methods_implemented = slot_def
                .proto_names
                .iter()
                .all(|name| implemented_protocols.contains(*name));

            if all_methods_implemented {
                // To use 'paired' def in priority, we remove used protocols.
                // For example, if add_radd is already used, we shouldn't use add and radd.
                for name in slot_def.proto_names {
                    implemented_protocols.remove(*name);
                }
            }

            all_methods_implemented
        })
    }
}

/// Represents a method registered as a normal method like `#[pymethods]`.
// TODO(kngwyu): Currently only __radd__-like methods use METH_COEXIST to prevent
// __add__-like methods from overriding them.
pub struct PyMethod {
    pub name: &'static str,
    pub proto: &'static str,
    pub can_coexist: bool,
}

impl PyMethod {
    const fn coexist(name: &'static str, proto: &'static str) -> Self {
        PyMethod {
            name,
            proto,
            can_coexist: true,
        }
    }
    const fn new(name: &'static str, proto: &'static str) -> Self {
        PyMethod {
            name,
            proto,
            can_coexist: false,
        }
    }
}

/// Represents a slot definition.
pub struct SlotDef {
    /// Protocols necessary to meet this def.
    /// E.g., we need `__setattr__` and `__delattr__` for invoking `set_setdelitem`.
    pub proto_names: &'static [&'static str],
    /// The Python slot name.
    pub slot: &'static str,
    /// The name of the function in pyo3 which implements the slot.
    pub slot_impl: &'static str,
}

impl SlotDef {
    const fn new(
        proto_names: &'static [&'static str],
        slot: &'static str,
        slot_impl: &'static str,
    ) -> Self {
        SlotDef {
            proto_names,
            slot,
            slot_impl,
        }
    }
}

pub const OBJECT: Proto = Proto {
    name: "Object",
    slots_trait: "pyo3::class::proto_methods::PyObjectProtocolSlots",
    slots_trait_slots: "object_protocol_slots",
    methods: &[
        MethodProto::new("__getattr__", "pyo3::class::basic::PyObjectGetAttrProtocol")
            .args(&["Name"])
            .has_self(),
        MethodProto::new("__setattr__", "pyo3::class::basic::PyObjectSetAttrProtocol")
            .args(&["Name", "Value"])
            .has_self(),
        MethodProto::new("__delattr__", "pyo3::class::basic::PyObjectDelAttrProtocol")
            .args(&["Name"])
            .has_self(),
        MethodProto::new("__str__", "pyo3::class::basic::PyObjectStrProtocol").has_self(),
        MethodProto::new("__repr__", "pyo3::class::basic::PyObjectReprProtocol").has_self(),
        MethodProto::new("__format__", "pyo3::class::basic::PyObjectFormatProtocol")
            .args(&["Format"])
            .has_self(),
        MethodProto::new("__hash__", "pyo3::class::basic::PyObjectHashProtocol").has_self(),
        MethodProto::new("__bytes__", "pyo3::class::basic::PyObjectBytesProtocol").has_self(),
        MethodProto::new("__richcmp__", "pyo3::class::basic::PyObjectRichcmpProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__bool__", "pyo3::class::basic::PyObjectBoolProtocol").has_self(),
    ],
    py_methods: &[
        PyMethod::new("__format__", "pyo3::class::basic::FormatProtocolImpl"),
        PyMethod::new("__bytes__", "pyo3::class::basic::BytesProtocolImpl"),
        PyMethod::new("__unicode__", "pyo3::class::basic::UnicodeProtocolImpl"),
    ],
    slot_defs: &[
        SlotDef::new(&["__str__"], "Py_tp_str", "pyo3::class::basic::str"),
        SlotDef::new(&["__repr__"], "Py_tp_repr", "pyo3::class::basic::repr"),
        SlotDef::new(&["__hash__"], "Py_tp_hash", "pyo3::class::basic::hash"),
        SlotDef::new(
            &["__getattr__"],
            "Py_tp_getattro",
            "pyo3::class::basic::getattr",
        ),
        SlotDef::new(
            &["__richcmp__"],
            "Py_tp_richcompare",
            "pyo3::class::basic::richcmp",
        ),
        SlotDef::new(
            &["__setattr__", "__delattr__"],
            "Py_tp_setattro",
            "pyo3::class::basic::setdelattr",
        ),
        SlotDef::new(
            &["__setattr__"],
            "Py_tp_setattro",
            "pyo3::class::basic::setattr",
        ),
        SlotDef::new(
            &["__delattr__"],
            "Py_tp_setattro",
            "pyo3::class::basic::delattr",
        ),
        SlotDef::new(&["__bool__"], "Py_nb_bool", "pyo3::class::basic::bool"),
    ],
};

pub const ASYNC: Proto = Proto {
    name: "Async",
    slots_trait: "pyo3::class::proto_methods::PyAsyncProtocolSlots",
    slots_trait_slots: "async_protocol_slots",
    methods: &[
        MethodProto::new("__await__", "pyo3::class::pyasync::PyAsyncAwaitProtocol")
            .args(&["Receiver"]),
        MethodProto::new("__aiter__", "pyo3::class::pyasync::PyAsyncAiterProtocol")
            .args(&["Receiver"]),
        MethodProto::new("__anext__", "pyo3::class::pyasync::PyAsyncAnextProtocol")
            .args(&["Receiver"]),
        MethodProto::new("__aenter__", "pyo3::class::pyasync::PyAsyncAenterProtocol").has_self(),
        MethodProto::new("__aexit__", "pyo3::class::pyasync::PyAsyncAexitProtocol")
            .args(&["ExcType", "ExcValue", "Traceback"])
            .has_self(),
    ],
    py_methods: &[
        PyMethod::new(
            "__aenter__",
            "pyo3::class::pyasync::PyAsyncAenterProtocolImpl",
        ),
        PyMethod::new(
            "__aexit__",
            "pyo3::class::pyasync::PyAsyncAexitProtocolImpl",
        ),
    ],
    slot_defs: &[
        SlotDef::new(
            &["__await__"],
            "Py_am_await",
            "pyo3::class::pyasync::await_",
        ),
        SlotDef::new(&["__aiter__"], "Py_am_aiter", "pyo3::class::pyasync::aiter"),
        SlotDef::new(&["__anext__"], "Py_am_anext", "pyo3::class::pyasync::anext"),
    ],
};

pub const BUFFER: Proto = Proto {
    name: "Buffer",
    slots_trait: "pyo3::class::proto_methods::PyBufferProtocolSlots",
    slots_trait_slots: "buffer_procs",
    methods: &[
        MethodProto::new(
            "bf_getbuffer",
            "pyo3::class::buffer::PyBufferGetBufferProtocol",
        )
        .has_self(),
        MethodProto::new(
            "bf_releasebuffer",
            "pyo3::class::buffer::PyBufferReleaseBufferProtocol",
        )
        .has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(
            &["bf_getbuffer"],
            "Py_bf_getbuffer",
            "pyo3::class::buffer::getbuffer",
        ),
        SlotDef::new(
            &["bf_releasebuffer"],
            "Py_bf_releasebuffer",
            "pyo3::class::buffer::releasebuffer",
        ),
    ],
};

pub const CONTEXT: Proto = Proto {
    name: "Context",
    slots_trait: "pyo3::class::proto_methods::PyContextProtocolSlots",
    slots_trait_slots: "context_protocol_slots",
    methods: &[
        MethodProto::new("__enter__", "pyo3::class::context::PyContextEnterProtocol").has_self(),
        MethodProto::new("__exit__", "pyo3::class::context::PyContextExitProtocol")
            .args(&["ExcType", "ExcValue", "Traceback"])
            .has_self(),
    ],
    py_methods: &[
        PyMethod::new(
            "__enter__",
            "pyo3::class::context::PyContextEnterProtocolImpl",
        ),
        PyMethod::new(
            "__exit__",
            "pyo3::class::context::PyContextExitProtocolImpl",
        ),
    ],
    slot_defs: &[],
};

pub const GC: Proto = Proto {
    name: "GC",
    slots_trait: "pyo3::class::proto_methods::PyGCProtocolSlots",
    slots_trait_slots: "gc_protocol_slots",
    methods: &[
        MethodProto::new("__traverse__", "pyo3::class::gc::PyGCTraverseProtocol")
            .has_self()
            .no_result(),
        MethodProto::new("__clear__", "pyo3::class::gc::PyGCClearProtocol")
            .has_self()
            .no_result(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(
            &["__traverse__"],
            "Py_tp_traverse",
            "pyo3::class::gc::traverse",
        ),
        SlotDef::new(&["__clear__"], "Py_tp_clear", "pyo3::class::gc::clear"),
    ],
};

pub const DESCR: Proto = Proto {
    name: "Descriptor",
    slots_trait: "pyo3::class::proto_methods::PyDescrProtocolSlots",
    slots_trait_slots: "descr_protocol_slots",
    methods: &[
        MethodProto::new("__get__", "pyo3::class::descr::PyDescrGetProtocol")
            .args(&["Receiver", "Inst", "Owner"]),
        MethodProto::new("__set__", "pyo3::class::descr::PyDescrSetProtocol")
            .args(&["Receiver", "Inst", "Value"]),
        MethodProto::new("__det__", "pyo3::class::descr::PyDescrDelProtocol")
            .args(&["Inst"])
            .has_self(),
        MethodProto::new("__set_name__", "pyo3::class::descr::PyDescrSetNameProtocol")
            .args(&["Inst"])
            .has_self(),
    ],
    py_methods: &[
        PyMethod::new("__del__", "pyo3::class::context::PyDescrDelProtocolImpl"),
        PyMethod::new(
            "__set_name__",
            "pyo3::class::context::PyDescrNameProtocolImpl",
        ),
    ],
    slot_defs: &[
        SlotDef::new(
            &["__get__"],
            "Py_tp_descr_get",
            "pyo3::class::descr::descr_get",
        ),
        SlotDef::new(
            &["__set__"],
            "Py_tp_descr_set",
            "pyo3::class::descr::descr_set",
        ),
    ],
};

pub const ITER: Proto = Proto {
    name: "Iter",
    slots_trait: "pyo3::class::proto_methods::PyIterProtocolSlots",
    slots_trait_slots: "iter_protocol_slots",
    py_methods: &[],
    methods: &[
        MethodProto::new("__iter__", "pyo3::class::iter::PyIterIterProtocol").args(&["Receiver"]),
        MethodProto::new("__next__", "pyo3::class::iter::PyIterNextProtocol").args(&["Receiver"]),
    ],
    slot_defs: &[
        SlotDef::new(&["__iter__"], "Py_tp_iter", "pyo3::class::iter::iter"),
        SlotDef::new(
            &["__next__"],
            "Py_tp_iternext",
            "pyo3::class::iter::iternext",
        ),
    ],
};

pub const MAPPING: Proto = Proto {
    name: "Mapping",
    slots_trait: "pyo3::class::proto_methods::PyMappingProtocolSlots",
    slots_trait_slots: "mapping_protocol_slots",
    methods: &[
        MethodProto::new("__len__", "pyo3::class::mapping::PyMappingLenProtocol").has_self(),
        MethodProto::new(
            "__getitem__",
            "pyo3::class::mapping::PyMappingGetItemProtocol",
        )
        .args(&["Key"])
        .has_self(),
        MethodProto::new(
            "__setitem__",
            "pyo3::class::mapping::PyMappingSetItemProtocol",
        )
        .args(&["Key", "Value"])
        .has_self(),
        MethodProto::new(
            "__delitem__",
            "pyo3::class::mapping::PyMappingDelItemProtocol",
        )
        .args(&["Key"])
        .has_self(),
        MethodProto::new(
            "__reversed__",
            "pyo3::class::mapping::PyMappingReversedProtocol",
        )
        .has_self(),
    ],
    py_methods: &[PyMethod::new(
        "__reversed__",
        "pyo3::class::mapping::PyMappingReversedProtocolImpl",
    )],
    slot_defs: &[
        SlotDef::new(&["__len__"], "Py_mp_length", "pyo3::class::mapping::len"),
        SlotDef::new(
            &["__getitem__"],
            "Py_mp_subscript",
            "pyo3::class::mapping::getitem",
        ),
        SlotDef::new(
            &["__setitem__", "__delitem__"],
            "Py_mp_ass_subscript",
            "pyo3::class::mapping::setdelitem",
        ),
        SlotDef::new(
            &["__setitem__"],
            "Py_mp_ass_subscript",
            "pyo3::class::mapping::setitem",
        ),
        SlotDef::new(
            &["__delitem__"],
            "Py_mp_ass_subscript",
            "pyo3::class::mapping::delitem",
        ),
    ],
};

pub const SEQ: Proto = Proto {
    name: "Sequence",
    slots_trait: "pyo3::class::proto_methods::PySequenceProtocolSlots",
    slots_trait_slots: "sequence_protocol_slots",
    methods: &[
        MethodProto::new("__len__", "pyo3::class::sequence::PySequenceLenProtocol").has_self(),
        MethodProto::new(
            "__getitem__",
            "pyo3::class::sequence::PySequenceGetItemProtocol",
        )
        .args(&["Index"])
        .has_self(),
        MethodProto::new(
            "__setitem__",
            "pyo3::class::sequence::PySequenceSetItemProtocol",
        )
        .args(&["Index", "Value"])
        .has_self(),
        MethodProto::new(
            "__delitem__",
            "pyo3::class::sequence::PySequenceDelItemProtocol",
        )
        .args(&["Index"])
        .has_self(),
        MethodProto::new(
            "__contains__",
            "pyo3::class::sequence::PySequenceContainsProtocol",
        )
        .args(&["Item"])
        .has_self(),
        MethodProto::new(
            "__concat__",
            "pyo3::class::sequence::PySequenceConcatProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__repeat__",
            "pyo3::class::sequence::PySequenceRepeatProtocol",
        )
        .args(&["Index"])
        .has_self(),
        MethodProto::new(
            "__inplace_concat__",
            "pyo3::class::sequence::PySequenceInplaceConcatProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__inplace_repeat__",
            "pyo3::class::sequence::PySequenceInplaceRepeatProtocol",
        )
        .args(&["Index"])
        .has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__len__"], "Py_sq_length", "pyo3::class::sequence::len"),
        SlotDef::new(
            &["__concat__"],
            "Py_sq_concat",
            "pyo3::class::sequence::concat",
        ),
        SlotDef::new(
            &["__repeat__"],
            "Py_sq_repeat",
            "pyo3::class::sequence::repeat",
        ),
        SlotDef::new(
            &["__getitem__"],
            "Py_sq_item",
            "pyo3::class::sequence::getitem",
        ),
        SlotDef::new(
            &["__setitem__", "__delitem__"],
            "Py_sq_ass_item",
            "pyo3::class::sequence::setdelitem",
        ),
        SlotDef::new(
            &["__setitem__"],
            "Py_sq_ass_item",
            "pyo3::class::sequence::setitem",
        ),
        SlotDef::new(
            &["__delitem__"],
            "Py_sq_ass_item",
            "pyo3::class::sequence::delitem",
        ),
        SlotDef::new(
            &["__contains__"],
            "Py_sq_contains",
            "pyo3::class::sequence::contains",
        ),
        SlotDef::new(
            &["__inplace_concat__"],
            "Py_sq_inplace_concat",
            "pyo3::class::sequence::inplace_concat",
        ),
        SlotDef::new(
            &["__inplace_repeat__"],
            "Py_sq_inplace_repeat",
            "pyo3::class::sequence::inplace_repeat",
        ),
    ],
};

pub const NUM: Proto = Proto {
    name: "Number",
    slots_trait: "pyo3::class::proto_methods::PyNumberProtocolSlots",
    slots_trait_slots: "number_protocol_slots",
    methods: &[
        MethodProto::new("__add__", "pyo3::class::number::PyNumberAddProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__sub__", "pyo3::class::number::PyNumberSubProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__mul__", "pyo3::class::number::PyNumberMulProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__matmul__", "pyo3::class::number::PyNumberMatmulProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new(
            "__truediv__",
            "pyo3::class::number::PyNumberTruedivProtocol",
        )
        .args(&["Left", "Right"]),
        MethodProto::new(
            "__floordiv__",
            "pyo3::class::number::PyNumberFloordivProtocol",
        )
        .args(&["Left", "Right"]),
        MethodProto::new("__mod__", "pyo3::class::number::PyNumberModProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__divmod__", "pyo3::class::number::PyNumberDivmodProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__pow__", "pyo3::class::number::PyNumberPowProtocol")
            .args(&["Left", "Right", "Modulo"]),
        MethodProto::new("__lshift__", "pyo3::class::number::PyNumberLShiftProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__rshift__", "pyo3::class::number::PyNumberRShiftProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__and__", "pyo3::class::number::PyNumberAndProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__xor__", "pyo3::class::number::PyNumberXorProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__or__", "pyo3::class::number::PyNumberOrProtocol")
            .args(&["Left", "Right"]),
        MethodProto::new("__radd__", "pyo3::class::number::PyNumberRAddProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rsub__", "pyo3::class::number::PyNumberRSubProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rmul__", "pyo3::class::number::PyNumberRMulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new(
            "__rmatmul__",
            "pyo3::class::number::PyNumberRMatmulProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__rtruediv__",
            "pyo3::class::number::PyNumberRTruedivProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__rfloordiv__",
            "pyo3::class::number::PyNumberRFloordivProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new("__rmod__", "pyo3::class::number::PyNumberRModProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new(
            "__rdivmod__",
            "pyo3::class::number::PyNumberRDivmodProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new("__rpow__", "pyo3::class::number::PyNumberRPowProtocol")
            .args(&["Other", "Modulo"])
            .has_self(),
        MethodProto::new(
            "__rlshift__",
            "pyo3::class::number::PyNumberRLShiftProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__rrshift__",
            "pyo3::class::number::PyNumberRRShiftProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new("__rand__", "pyo3::class::number::PyNumberRAndProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rxor__", "pyo3::class::number::PyNumberRXorProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ror__", "pyo3::class::number::PyNumberROrProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__iadd__", "pyo3::class::number::PyNumberIAddProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__isub__", "pyo3::class::number::PyNumberISubProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__imul__", "pyo3::class::number::PyNumberIMulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new(
            "__imatmul__",
            "pyo3::class::number::PyNumberIMatmulProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__itruediv__",
            "pyo3::class::number::PyNumberITruedivProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__ifloordiv__",
            "pyo3::class::number::PyNumberIFloordivProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new("__imod__", "pyo3::class::number::PyNumberIModProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ipow__", "pyo3::class::number::PyNumberIPowProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new(
            "__ilshift__",
            "pyo3::class::number::PyNumberILShiftProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new(
            "__irshift__",
            "pyo3::class::number::PyNumberIRShiftProtocol",
        )
        .args(&["Other"])
        .has_self(),
        MethodProto::new("__iand__", "pyo3::class::number::PyNumberIAndProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ixor__", "pyo3::class::number::PyNumberIXorProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ior__", "pyo3::class::number::PyNumberIOrProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__neg__", "pyo3::class::number::PyNumberNegProtocol").has_self(),
        MethodProto::new("__pos__", "pyo3::class::number::PyNumberPosProtocol").has_self(),
        MethodProto::new("__abs__", "pyo3::class::number::PyNumberAbsProtocol").has_self(),
        MethodProto::new("__invert__", "pyo3::class::number::PyNumberInvertProtocol").has_self(),
        MethodProto::new(
            "__complex__",
            "pyo3::class::number::PyNumberComplexProtocol",
        )
        .has_self(),
        MethodProto::new("__int__", "pyo3::class::number::PyNumberIntProtocol").has_self(),
        MethodProto::new("__float__", "pyo3::class::number::PyNumberFloatProtocol").has_self(),
        MethodProto::new("__index__", "pyo3::class::number::PyNumberIndexProtocol").has_self(),
        MethodProto::new("__round__", "pyo3::class::number::PyNumberRoundProtocol")
            .args(&["NDigits"])
            .has_self(),
    ],
    py_methods: &[
        PyMethod::coexist("__radd__", "pyo3::class::number::PyNumberRAddProtocolImpl"),
        PyMethod::coexist("__rsub__", "pyo3::class::number::PyNumberRSubProtocolImpl"),
        PyMethod::coexist("__rmul__", "pyo3::class::number::PyNumberRMulProtocolImpl"),
        PyMethod::coexist(
            "__rmatmul__",
            "pyo3::class::number::PyNumberRMatmulProtocolImpl",
        ),
        PyMethod::coexist(
            "__rtruediv__",
            "pyo3::class::number::PyNumberRTruedivProtocolImpl",
        ),
        PyMethod::coexist(
            "__rfloordiv__",
            "pyo3::class::number::PyNumberRFloordivProtocolImpl",
        ),
        PyMethod::coexist("__rmod__", "pyo3::class::number::PyNumberRModProtocolImpl"),
        PyMethod::coexist(
            "__rdivmod__",
            "pyo3::class::number::PyNumberRDivmodProtocolImpl",
        ),
        PyMethod::coexist("__rpow__", "pyo3::class::number::PyNumberRPowProtocolImpl"),
        PyMethod::coexist(
            "__rlshift__",
            "pyo3::class::number::PyNumberRLShiftProtocolImpl",
        ),
        PyMethod::coexist(
            "__rrshift__",
            "pyo3::class::number::PyNumberRRShiftProtocolImpl",
        ),
        PyMethod::coexist("__rand__", "pyo3::class::number::PyNumberRAndProtocolImpl"),
        PyMethod::coexist("__rxor__", "pyo3::class::number::PyNumberRXorProtocolImpl"),
        PyMethod::coexist("__ror__", "pyo3::class::number::PyNumberROrProtocolImpl"),
        PyMethod::new(
            "__complex__",
            "pyo3::class::number::PyNumberComplexProtocolImpl",
        ),
        PyMethod::new(
            "__round__",
            "pyo3::class::number::PyNumberRoundProtocolImpl",
        ),
    ],
    slot_defs: &[
        SlotDef::new(
            &["__add__", "__radd__"],
            "Py_nb_add",
            "pyo3::class::number::add_radd",
        ),
        SlotDef::new(&["__add__"], "Py_nb_add", "pyo3::class::number::add"),
        SlotDef::new(&["__radd__"], "Py_nb_add", "pyo3::class::number::radd"),
        SlotDef::new(
            &["__sub__", "__rsub__"],
            "Py_nb_subtract",
            "pyo3::class::number::sub_rsub",
        ),
        SlotDef::new(&["__sub__"], "Py_nb_subtract", "pyo3::class::number::sub"),
        SlotDef::new(&["__rsub__"], "Py_nb_subtract", "pyo3::class::number::rsub"),
        SlotDef::new(
            &["__mul__", "__rmul__"],
            "Py_nb_multiply",
            "pyo3::class::number::mul_rmul",
        ),
        SlotDef::new(&["__mul__"], "Py_nb_multiply", "pyo3::class::number::mul"),
        SlotDef::new(&["__rmul__"], "Py_nb_multiply", "pyo3::class::number::rmul"),
        SlotDef::new(&["__mod__"], "Py_nb_remainder", "pyo3::class::number::mod_"),
        SlotDef::new(
            &["__divmod__", "__rdivmod__"],
            "Py_nb_divmod",
            "pyo3::class::number::divmod_rdivmod",
        ),
        SlotDef::new(
            &["__divmod__"],
            "Py_nb_divmod",
            "pyo3::class::number::divmod",
        ),
        SlotDef::new(
            &["__rdivmod__"],
            "Py_nb_divmod",
            "pyo3::class::number::rdivmod",
        ),
        SlotDef::new(
            &["__pow__", "__rpow__"],
            "Py_nb_power",
            "pyo3::class::number::pow_rpow",
        ),
        SlotDef::new(&["__pow__"], "Py_nb_power", "pyo3::class::number::pow"),
        SlotDef::new(&["__rpow__"], "Py_nb_power", "pyo3::class::number::rpow"),
        SlotDef::new(&["__neg__"], "Py_nb_negative", "pyo3::class::number::neg"),
        SlotDef::new(&["__pos__"], "Py_nb_positive", "pyo3::class::number::pos"),
        SlotDef::new(&["__abs__"], "Py_nb_absolute", "pyo3::class::number::abs"),
        SlotDef::new(
            &["__invert__"],
            "Py_nb_invert",
            "pyo3::class::number::invert",
        ),
        SlotDef::new(
            &["__lshift__", "__rlshift__"],
            "Py_nb_lshift",
            "pyo3::class::number::lshift_rlshift",
        ),
        SlotDef::new(
            &["__lshift__"],
            "Py_nb_lshift",
            "pyo3::class::number::lshift",
        ),
        SlotDef::new(
            &["__rlshift__"],
            "Py_nb_lshift",
            "pyo3::class::number::rlshift",
        ),
        SlotDef::new(
            &["__rshift__", "__rrshift__"],
            "Py_nb_rshift",
            "pyo3::class::number::rshift_rrshift",
        ),
        SlotDef::new(
            &["__rshift__"],
            "Py_nb_rshift",
            "pyo3::class::number::rshift",
        ),
        SlotDef::new(
            &["__rrshift__"],
            "Py_nb_rshift",
            "pyo3::class::number::rrshift",
        ),
        SlotDef::new(
            &["__and__", "__rand__"],
            "Py_nb_and",
            "pyo3::class::number::and_rand",
        ),
        SlotDef::new(&["__and__"], "Py_nb_and", "pyo3::class::number::and"),
        SlotDef::new(&["__rand__"], "Py_nb_and", "pyo3::class::number::rand"),
        SlotDef::new(
            &["__xor__", "__rxor__"],
            "Py_nb_xor",
            "pyo3::class::number::xor_rxor",
        ),
        SlotDef::new(&["__xor__"], "Py_nb_xor", "pyo3::class::number::xor"),
        SlotDef::new(&["__rxor__"], "Py_nb_xor", "pyo3::class::number::rxor"),
        SlotDef::new(
            &["__or__", "__ror__"],
            "Py_nb_or",
            "pyo3::class::number::or_ror",
        ),
        SlotDef::new(&["__or__"], "Py_nb_or", "pyo3::class::number::or"),
        SlotDef::new(&["__ror__"], "Py_nb_or", "pyo3::class::number::ror"),
        SlotDef::new(&["__int__"], "Py_nb_int", "pyo3::class::number::int"),
        SlotDef::new(&["__float__"], "Py_nb_float", "pyo3::class::number::float"),
        SlotDef::new(
            &["__iadd__"],
            "Py_nb_inplace_add",
            "pyo3::class::number::iadd",
        ),
        SlotDef::new(
            &["__isub__"],
            "Py_nb_inplace_subtract",
            "pyo3::class::number::isub",
        ),
        SlotDef::new(
            &["__imul__"],
            "Py_nb_inplace_multiply",
            "pyo3::class::number::imul",
        ),
        SlotDef::new(
            &["__imod__"],
            "Py_nb_inplace_remainder",
            "pyo3::class::number::imod",
        ),
        SlotDef::new(
            &["__ipow__"],
            "Py_nb_inplace_power",
            "pyo3::class::number::ipow",
        ),
        SlotDef::new(
            &["__ilshift__"],
            "Py_nb_inplace_lshift",
            "pyo3::class::number::ilshift",
        ),
        SlotDef::new(
            &["__irshift__"],
            "Py_nb_inplace_rshift",
            "pyo3::class::number::irshift",
        ),
        SlotDef::new(
            &["__iand__"],
            "Py_nb_inplace_and",
            "pyo3::class::number::iand",
        ),
        SlotDef::new(
            &["__ixor__"],
            "Py_nb_inplace_xor",
            "pyo3::class::number::ixor",
        ),
        SlotDef::new(&["__ior__"], "Py_nb_inplace_or", "pyo3::class::number::ior"),
        SlotDef::new(
            &["__floordiv__", "__rfloordiv__"],
            "Py_nb_floor_divide",
            "pyo3::class::number::floordiv_rfloordiv",
        ),
        SlotDef::new(
            &["__floordiv__"],
            "Py_nb_floor_divide",
            "pyo3::class::number::floordiv",
        ),
        SlotDef::new(
            &["__rfloordiv__"],
            "Py_nb_floor_divide",
            "pyo3::class::number::rfloordiv",
        ),
        SlotDef::new(
            &["__truediv__", "__rtruediv__"],
            "Py_nb_true_divide",
            "pyo3::class::number::truediv_rtruediv",
        ),
        SlotDef::new(
            &["__truediv__"],
            "Py_nb_true_divide",
            "pyo3::class::number::truediv",
        ),
        SlotDef::new(
            &["__rtruediv__"],
            "Py_nb_true_divide",
            "pyo3::class::number::rtruediv",
        ),
        SlotDef::new(
            &["__ifloordiv__"],
            "Py_nb_inplace_floor_divide",
            "pyo3::class::number::ifloordiv",
        ),
        SlotDef::new(
            &["__itruediv__"],
            "Py_nb_inplace_true_divide",
            "pyo3::class::number::itruediv",
        ),
        SlotDef::new(&["__index__"], "Py_nb_index", "pyo3::class::number::index"),
        SlotDef::new(
            &["__matmul__", "__rmatmul__"],
            "Py_nb_matrix_multiply",
            "pyo3::class::number::matmul_rmatmul",
        ),
        SlotDef::new(
            &["__matmul__"],
            "Py_nb_matrix_multiply",
            "pyo3::class::number::matmul",
        ),
        SlotDef::new(
            &["__rmatmul__"],
            "Py_nb_matrix_multiply",
            "pyo3::class::number::rmatmul",
        ),
        SlotDef::new(
            &["__imatmul__"],
            "Py_nb_inplace_matrix_multiply",
            "pyo3::class::number::imatmul",
        ),
    ],
};
