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
        self.methods.iter().find(|m| query == m.name())
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
        MethodProto::Binary {
            name: "__getattr__",
            arg: "Name",
            proto: "pyo3::class::basic::PyObjectGetAttrProtocol",
        },
        MethodProto::Ternary {
            name: "__setattr__",
            arg1: "Name",
            arg2: "Value",
            proto: "pyo3::class::basic::PyObjectSetAttrProtocol",
        },
        MethodProto::Binary {
            name: "__delattr__",
            arg: "Name",
            proto: "pyo3::class::basic::PyObjectDelAttrProtocol",
        },
        MethodProto::Unary {
            name: "__str__",
            proto: "pyo3::class::basic::PyObjectStrProtocol",
        },
        MethodProto::Unary {
            name: "__repr__",
            proto: "pyo3::class::basic::PyObjectReprProtocol",
        },
        MethodProto::Binary {
            name: "__format__",
            arg: "Format",
            proto: "pyo3::class::basic::PyObjectFormatProtocol",
        },
        MethodProto::Unary {
            name: "__hash__",
            proto: "pyo3::class::basic::PyObjectHashProtocol",
        },
        MethodProto::Unary {
            name: "__bytes__",
            proto: "pyo3::class::basic::PyObjectBytesProtocol",
        },
        MethodProto::Binary {
            name: "__richcmp__",
            arg: "Other",
            proto: "pyo3::class::basic::PyObjectRichcmpProtocol",
        },
        MethodProto::Unary {
            name: "__bool__",
            proto: "pyo3::class::basic::PyObjectBoolProtocol",
        },
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
        SlotGetter::new(&["__richcmp__"], "get_richcompare"),
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
        MethodProto::UnaryS {
            name: "__await__",
            arg: "Receiver",
            proto: "pyo3::class::pyasync::PyAsyncAwaitProtocol",
        },
        MethodProto::UnaryS {
            name: "__aiter__",
            arg: "Receiver",
            proto: "pyo3::class::pyasync::PyAsyncAiterProtocol",
        },
        MethodProto::UnaryS {
            name: "__anext__",
            arg: "Receiver",
            proto: "pyo3::class::pyasync::PyAsyncAnextProtocol",
        },
        MethodProto::Unary {
            name: "__aenter__",
            proto: "pyo3::class::pyasync::PyAsyncAenterProtocol",
        },
        MethodProto::Quaternary {
            name: "__aexit__",
            arg1: "ExcType",
            arg2: "ExcValue",
            arg3: "Traceback",
            proto: "pyo3::class::pyasync::PyAsyncAexitProtocol",
        },
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
        MethodProto::Unary {
            name: "bf_getbuffer",
            proto: "pyo3::class::buffer::PyBufferGetBufferProtocol",
        },
        MethodProto::Unary {
            name: "bf_releasebuffer",
            proto: "pyo3::class::buffer::PyBufferReleaseBufferProtocol",
        },
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
        MethodProto::Unary {
            name: "__enter__",
            proto: "pyo3::class::context::PyContextEnterProtocol",
        },
        MethodProto::Quaternary {
            name: "__exit__",
            arg1: "ExcType",
            arg2: "ExcValue",
            arg3: "Traceback",
            proto: "pyo3::class::context::PyContextExitProtocol",
        },
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
        MethodProto::Free {
            name: "__traverse__",
            proto: "pyo3::class::gc::PyGCTraverseProtocol",
        },
        MethodProto::Free {
            name: "__clear__",
            proto: "pyo3::class::gc::PyGCClearProtocol",
        },
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
        MethodProto::TernaryS {
            name: "__get__",
            arg1: "Receiver",
            arg2: "Inst",
            arg3: "Owner",
            proto: "pyo3::class::descr::PyDescrGetProtocol",
        },
        MethodProto::TernaryS {
            name: "__set__",
            arg1: "Receiver",
            arg2: "Inst",
            arg3: "Value",
            proto: "pyo3::class::descr::PyDescrSetProtocol",
        },
        MethodProto::Binary {
            name: "__det__",
            arg: "Inst",
            proto: "pyo3::class::descr::PyDescrDelProtocol",
        },
        MethodProto::Binary {
            name: "__set_name__",
            arg: "Inst",
            proto: "pyo3::class::descr::PyDescrSetNameProtocol",
        },
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
        MethodProto::UnaryS {
            name: "__iter__",
            arg: "Receiver",
            proto: "pyo3::class::iter::PyIterIterProtocol",
        },
        MethodProto::UnaryS {
            name: "__next__",
            arg: "Receiver",
            proto: "pyo3::class::iter::PyIterNextProtocol",
        },
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
        MethodProto::Unary {
            name: "__len__",
            proto: "pyo3::class::mapping::PyMappingLenProtocol",
        },
        MethodProto::Binary {
            name: "__getitem__",
            arg: "Key",
            proto: "pyo3::class::mapping::PyMappingGetItemProtocol",
        },
        MethodProto::Ternary {
            name: "__setitem__",
            arg1: "Key",
            arg2: "Value",
            proto: "pyo3::class::mapping::PyMappingSetItemProtocol",
        },
        MethodProto::Binary {
            name: "__delitem__",
            arg: "Key",
            proto: "pyo3::class::mapping::PyMappingDelItemProtocol",
        },
        MethodProto::Unary {
            name: "__reversed__",
            proto: "pyo3::class::mapping::PyMappingReversedProtocol",
        },
    ],
    py_methods: &[PyMethod::new(
        "__reversed__",
        "pyo3::class::mapping::PyMappingReversedProtocolImpl",
    )],
    slot_getters: &[
        SlotGetter::new(&["__len__"], "get_length"),
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
        MethodProto::Unary {
            name: "__len__",
            proto: "pyo3::class::sequence::PySequenceLenProtocol",
        },
        MethodProto::Binary {
            name: "__getitem__",
            arg: "Index",
            proto: "pyo3::class::sequence::PySequenceGetItemProtocol",
        },
        MethodProto::Ternary {
            name: "__setitem__",
            arg1: "Index",
            arg2: "Value",
            proto: "pyo3::class::sequence::PySequenceSetItemProtocol",
        },
        MethodProto::Binary {
            name: "__delitem__",
            arg: "Index",
            proto: "pyo3::class::sequence::PySequenceDelItemProtocol",
        },
        MethodProto::Binary {
            name: "__contains__",
            arg: "Item",
            proto: "pyo3::class::sequence::PySequenceContainsProtocol",
        },
        MethodProto::Binary {
            name: "__concat__",
            arg: "Other",
            proto: "pyo3::class::sequence::PySequenceConcatProtocol",
        },
        MethodProto::Binary {
            name: "__repeat__",
            arg: "Index",
            proto: "pyo3::class::sequence::PySequenceRepeatProtocol",
        },
        MethodProto::Binary {
            name: "__inplace_concat__",
            arg: "Other",
            proto: "pyo3::class::sequence::PySequenceInplaceConcatProtocol",
        },
        MethodProto::Binary {
            name: "__inplace_repeat__",
            arg: "Index",
            proto: "pyo3::class::sequence::PySequenceInplaceRepeatProtocol",
        },
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
        MethodProto::BinaryS {
            name: "__add__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberAddProtocol",
        },
        MethodProto::BinaryS {
            name: "__sub__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberSubProtocol",
        },
        MethodProto::BinaryS {
            name: "__mul__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberMulProtocol",
        },
        MethodProto::BinaryS {
            name: "__matmul__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberMatmulProtocol",
        },
        MethodProto::BinaryS {
            name: "__truediv__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberTruedivProtocol",
        },
        MethodProto::BinaryS {
            name: "__floordiv__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberFloordivProtocol",
        },
        MethodProto::BinaryS {
            name: "__mod__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberModProtocol",
        },
        MethodProto::BinaryS {
            name: "__divmod__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberDivmodProtocol",
        },
        MethodProto::TernaryS {
            name: "__pow__",
            arg1: "Left",
            arg2: "Right",
            arg3: "Modulo",
            proto: "pyo3::class::number::PyNumberPowProtocol",
        },
        MethodProto::BinaryS {
            name: "__lshift__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberLShiftProtocol",
        },
        MethodProto::BinaryS {
            name: "__rshift__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberRShiftProtocol",
        },
        MethodProto::BinaryS {
            name: "__and__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberAndProtocol",
        },
        MethodProto::BinaryS {
            name: "__xor__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberXorProtocol",
        },
        MethodProto::BinaryS {
            name: "__or__",
            arg1: "Left",
            arg2: "Right",
            proto: "pyo3::class::number::PyNumberOrProtocol",
        },
        MethodProto::Binary {
            name: "__radd__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRAddProtocol",
        },
        MethodProto::Binary {
            name: "__rsub__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRSubProtocol",
        },
        MethodProto::Binary {
            name: "__rmul__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRMulProtocol",
        },
        MethodProto::Binary {
            name: "__rmatmul__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRMatmulProtocol",
        },
        MethodProto::Binary {
            name: "__rtruediv__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRTruedivProtocol",
        },
        MethodProto::Binary {
            name: "__rfloordiv__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRFloordivProtocol",
        },
        MethodProto::Binary {
            name: "__rmod__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRModProtocol",
        },
        MethodProto::Binary {
            name: "__rdivmod__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRDivmodProtocol",
        },
        MethodProto::Ternary {
            name: "__rpow__",
            arg1: "Other",
            arg2: "Modulo",
            proto: "pyo3::class::number::PyNumberRPowProtocol",
        },
        MethodProto::Binary {
            name: "__rlshift__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRLShiftProtocol",
        },
        MethodProto::Binary {
            name: "__rrshift__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRRShiftProtocol",
        },
        MethodProto::Binary {
            name: "__rand__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRAndProtocol",
        },
        MethodProto::Binary {
            name: "__rxor__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberRXorProtocol",
        },
        MethodProto::Binary {
            name: "__ror__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberROrProtocol",
        },
        MethodProto::Binary {
            name: "__iadd__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIAddProtocol",
        },
        MethodProto::Binary {
            name: "__isub__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberISubProtocol",
        },
        MethodProto::Binary {
            name: "__imul__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIMulProtocol",
        },
        MethodProto::Binary {
            name: "__imatmul__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIMatmulProtocol",
        },
        MethodProto::Binary {
            name: "__itruediv__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberITruedivProtocol",
        },
        MethodProto::Binary {
            name: "__ifloordiv__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIFloordivProtocol",
        },
        MethodProto::Binary {
            name: "__imod__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIModProtocol",
        },
        MethodProto::Binary {
            name: "__ipow__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIPowProtocol",
        },
        MethodProto::Binary {
            name: "__ilshift__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberILShiftProtocol",
        },
        MethodProto::Binary {
            name: "__irshift__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIRShiftProtocol",
        },
        MethodProto::Binary {
            name: "__iand__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIAndProtocol",
        },
        MethodProto::Binary {
            name: "__ixor__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIXorProtocol",
        },
        MethodProto::Binary {
            name: "__ior__",
            arg: "Other",
            proto: "pyo3::class::number::PyNumberIOrProtocol",
        },
        MethodProto::Unary {
            name: "__neg__",
            proto: "pyo3::class::number::PyNumberNegProtocol",
        },
        MethodProto::Unary {
            name: "__pos__",
            proto: "pyo3::class::number::PyNumberPosProtocol",
        },
        MethodProto::Unary {
            name: "__abs__",
            proto: "pyo3::class::number::PyNumberAbsProtocol",
        },
        MethodProto::Unary {
            name: "__invert__",
            proto: "pyo3::class::number::PyNumberInvertProtocol",
        },
        MethodProto::Unary {
            name: "__complex__",
            proto: "pyo3::class::number::PyNumberComplexProtocol",
        },
        MethodProto::Unary {
            name: "__int__",
            proto: "pyo3::class::number::PyNumberIntProtocol",
        },
        MethodProto::Unary {
            name: "__float__",
            proto: "pyo3::class::number::PyNumberFloatProtocol",
        },
        MethodProto::Unary {
            name: "__index__",
            proto: "pyo3::class::number::PyNumberIndexProtocol",
        },
        MethodProto::Binary {
            name: "__round__",
            arg: "NDigits",
            proto: "pyo3::class::number::PyNumberRoundProtocol",
        },
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
