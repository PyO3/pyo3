// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::proto_method::MethodProto;
use std::collections::HashSet;

/// Predicates for `#[pyproto]`.
pub struct Proto {
    /// The name of this protocol. E.g., Iter.
    pub name: &'static str,
    /// Extension trait that has `get_*` methods
    pub extension_trait: &'static str,
    /// All methods.
    pub methods: &'static [MethodProto],
    /// All methods registered as normal methods like `#[pymethods]`.
    pub py_methods: &'static [PyMethod],
    /// All methods registered to the slot table.
    slot_getters: &'static [SlotGetter],
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
    pub(crate) fn slot_getters(
        &self,
        mut implemented_protocols: HashSet<String>,
    ) -> impl Iterator<Item = &'static str> {
        self.slot_getters.iter().filter_map(move |getter| {
            // If any required method is not implemented, we skip this setter.
            if getter
                .proto_names
                .iter()
                .any(|name| !implemented_protocols.contains(*name))
            {
                return None;
            }
            // To use 'paired' setter in priority, we remove used protocols.
            // For example, if set_add_radd is already used, we shouldn't use set_add and set_radd.
            for name in getter.proto_names {
                implemented_protocols.remove(*name);
            }
            Some(getter.get_function)
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

/// Represents a setter used to register a method to the method table.
struct SlotGetter {
    /// Protocols necessary for invoking this setter.
    /// E.g., we need `__setattr__` and `__delattr__` for invoking `set_setdelitem`.
    pub proto_names: &'static [&'static str],
    /// The name of the setter called to the method table.
    pub get_function: &'static str,
}

impl SlotGetter {
    const fn new(names: &'static [&'static str], get_function: &'static str) -> Self {
        SlotGetter {
            proto_names: names,
            get_function,
        }
    }
}

pub const OBJECT: Proto = Proto {
    name: "Object",
    extension_trait: "pyo3::class::basic::PyBasicSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__str__"], "get_str"),
        SlotGetter::new(&["__repr__"], "get_repr"),
        SlotGetter::new(&["__hash__"], "get_hash"),
        SlotGetter::new(&["__getattr__"], "get_getattr"),
        SlotGetter::new(&["__richcmp__"], "get_richcmp"),
        SlotGetter::new(&["__setattr__", "__delattr__"], "get_setdelattr"),
        SlotGetter::new(&["__setattr__"], "get_setattr"),
        SlotGetter::new(&["__delattr__"], "get_delattr"),
        SlotGetter::new(&["__bool__"], "get_bool"),
    ],
};

pub const ASYNC: Proto = Proto {
    name: "Async",
    extension_trait: "pyo3::class::pyasync::PyAsyncSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__await__"], "get_await"),
        SlotGetter::new(&["__aiter__"], "get_aiter"),
        SlotGetter::new(&["__anext__"], "get_anext"),
    ],
};

pub const BUFFER: Proto = Proto {
    name: "Buffer",
    extension_trait: "pyo3::class::buffer::PyBufferSlots",
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
    slot_getters: &[
        SlotGetter::new(&["bf_getbuffer"], "get_getbuffer"),
        SlotGetter::new(&["bf_releasebuffer"], "get_releasebuffer"),
    ],
};

pub const CONTEXT: Proto = Proto {
    name: "Context",
    extension_trait: "",
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
    slot_getters: &[],
};

pub const GC: Proto = Proto {
    name: "GC",
    extension_trait: "pyo3::class::gc::PyGCSlots",
    methods: &[
        MethodProto::new("__traverse__", "pyo3::class::gc::PyGCTraverseProtocol")
            .has_self()
            .no_result(),
        MethodProto::new("__clear__", "pyo3::class::gc::PyGCClearProtocol")
            .has_self()
            .no_result(),
    ],
    py_methods: &[],
    slot_getters: &[
        SlotGetter::new(&["__traverse__"], "get_traverse"),
        SlotGetter::new(&["__clear__"], "get_clear"),
    ],
};

pub const DESCR: Proto = Proto {
    name: "Descriptor",
    extension_trait: "pyo3::class::descr::PyDescrSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__get__"], "get_descr_get"),
        SlotGetter::new(&["__set__"], "get_descr_set"),
    ],
};

pub const ITER: Proto = Proto {
    name: "Iter",
    extension_trait: "pyo3::class::iter::PyIterSlots",
    py_methods: &[],
    methods: &[
        MethodProto::new("__iter__", "pyo3::class::iter::PyIterIterProtocol").args(&["Receiver"]),
        MethodProto::new("__next__", "pyo3::class::iter::PyIterNextProtocol").args(&["Receiver"]),
    ],
    slot_getters: &[
        SlotGetter::new(&["__iter__"], "get_iter"),
        SlotGetter::new(&["__next__"], "get_iternext"),
    ],
};

pub const MAPPING: Proto = Proto {
    name: "Mapping",
    extension_trait: "pyo3::class::mapping::PyMappingSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__len__"], "get_len"),
        SlotGetter::new(&["__getitem__"], "get_getitem"),
        SlotGetter::new(&["__setitem__", "__delitem__"], "get_setdelitem"),
        SlotGetter::new(&["__setitem__"], "get_setitem"),
        SlotGetter::new(&["__delitem__"], "get_delitem"),
    ],
};

pub const SEQ: Proto = Proto {
    name: "Sequence",
    extension_trait: "pyo3::class::sequence::PySequenceSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__len__"], "get_len"),
        SlotGetter::new(&["__concat__"], "get_concat"),
        SlotGetter::new(&["__repeat__"], "get_repeat"),
        SlotGetter::new(&["__getitem__"], "get_getitem"),
        SlotGetter::new(&["__setitem__", "__delitem__"], "get_setdelitem"),
        SlotGetter::new(&["__setitem__"], "get_setitem"),
        SlotGetter::new(&["__delitem__"], "get_delitem"),
        SlotGetter::new(&["__contains__"], "get_contains"),
        SlotGetter::new(&["__inplace_concat__"], "get_inplace_concat"),
        SlotGetter::new(&["__inplace_repeat__"], "get_inplace_repeat"),
    ],
};

pub const NUM: Proto = Proto {
    name: "Number",
    extension_trait: "pyo3::class::number::PyNumberSlots",
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
    slot_getters: &[
        SlotGetter::new(&["__add__", "__radd__"], "get_add_radd"),
        SlotGetter::new(&["__add__"], "get_add"),
        SlotGetter::new(&["__radd__"], "get_radd"),
        SlotGetter::new(&["__sub__", "__rsub__"], "get_sub_rsub"),
        SlotGetter::new(&["__sub__"], "get_sub"),
        SlotGetter::new(&["__rsub__"], "get_rsub"),
        SlotGetter::new(&["__mul__", "__rmul__"], "get_mul_rmul"),
        SlotGetter::new(&["__mul__"], "get_mul"),
        SlotGetter::new(&["__rmul__"], "get_rmul"),
        SlotGetter::new(&["__mod__"], "get_mod"),
        SlotGetter::new(&["__divmod__", "__rdivmod__"], "get_divmod_rdivmod"),
        SlotGetter::new(&["__divmod__"], "get_divmod"),
        SlotGetter::new(&["__rdivmod__"], "get_rdivmod"),
        SlotGetter::new(&["__pow__", "__rpow__"], "get_pow_rpow"),
        SlotGetter::new(&["__pow__"], "get_pow"),
        SlotGetter::new(&["__rpow__"], "get_rpow"),
        SlotGetter::new(&["__neg__"], "get_neg"),
        SlotGetter::new(&["__pos__"], "get_pos"),
        SlotGetter::new(&["__abs__"], "get_abs"),
        SlotGetter::new(&["__invert__"], "get_invert"),
        SlotGetter::new(&["__lshift__", "__rlshift__"], "get_lshift_rlshift"),
        SlotGetter::new(&["__lshift__"], "get_lshift"),
        SlotGetter::new(&["__rlshift__"], "get_rlshift"),
        SlotGetter::new(&["__rshift__", "__rrshift__"], "get_rshift_rrshift"),
        SlotGetter::new(&["__rshift__"], "get_rshift"),
        SlotGetter::new(&["__rrshift__"], "get_rrshift"),
        SlotGetter::new(&["__and__", "__rand__"], "get_and_rand"),
        SlotGetter::new(&["__and__"], "get_and"),
        SlotGetter::new(&["__rand__"], "get_rand"),
        SlotGetter::new(&["__xor__", "__rxor__"], "get_xor_rxor"),
        SlotGetter::new(&["__xor__"], "get_xor"),
        SlotGetter::new(&["__rxor__"], "get_rxor"),
        SlotGetter::new(&["__or__", "__ror__"], "get_or_ror"),
        SlotGetter::new(&["__or__"], "get_or"),
        SlotGetter::new(&["__ror__"], "get_ror"),
        SlotGetter::new(&["__int__"], "get_int"),
        SlotGetter::new(&["__float__"], "get_float"),
        SlotGetter::new(&["__iadd__"], "get_iadd"),
        SlotGetter::new(&["__isub__"], "get_isub"),
        SlotGetter::new(&["__imul__"], "get_imul"),
        SlotGetter::new(&["__imod__"], "get_imod"),
        SlotGetter::new(&["__ipow__"], "get_ipow"),
        SlotGetter::new(&["__ilshift__"], "get_ilshift"),
        SlotGetter::new(&["__irshift__"], "get_irshift"),
        SlotGetter::new(&["__iand__"], "get_iand"),
        SlotGetter::new(&["__ixor__"], "get_ixor"),
        SlotGetter::new(&["__ior__"], "get_ior"),
        SlotGetter::new(&["__floordiv__", "__rfloordiv__"], "get_floordiv_rfloordiv"),
        SlotGetter::new(&["__floordiv__"], "get_floordiv"),
        SlotGetter::new(&["__rfloordiv__"], "get_rfloordiv"),
        SlotGetter::new(&["__truediv__", "__rtruediv__"], "get_truediv_rtruediv"),
        SlotGetter::new(&["__truediv__"], "get_truediv"),
        SlotGetter::new(&["__rtruediv__"], "get_rtruediv"),
        SlotGetter::new(&["__ifloordiv__"], "get_ifloordiv"),
        SlotGetter::new(&["__itruediv__"], "get_itruediv"),
        SlotGetter::new(&["__index__"], "get_index"),
        SlotGetter::new(&["__matmul__", "__rmatmul__"], "get_matmul_rmatmul"),
        SlotGetter::new(&["__matmul__"], "get_matmul"),
        SlotGetter::new(&["__rmatmul__"], "get_rmatmul"),
        SlotGetter::new(&["__imatmul__"], "get_imatmul"),
    ],
};
