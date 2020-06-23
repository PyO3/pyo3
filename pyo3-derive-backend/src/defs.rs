// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::func::MethodProto;

/// Predicates for `#[pyproto]`.
pub struct Proto {
    /// The name of this protocol. E.g., Iter.
    pub name: &'static str,
    /// The name of slot table. E.g., PyIterMethods.
    pub slot_table: &'static str,
    /// The name of the setter used to set the table to `PyProtoRegistry`.
    pub set_slot_table: &'static str,
    /// All methods.
    pub methods: &'static [MethodProto],
    /// All methods registered as normal methods like `#[pymethods]`.
    pub py_methods: &'static [PyMethod],
    /// All methods registered to the slot table.
    pub slot_setters: &'static [SlotSetter],
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
pub struct SlotSetter {
    /// Protocols necessary for invoking this setter.
    /// E.g., we need `__setattr__` and `__delattr__` for invoking `set_setdelitem`.
    pub proto_names: &'static [&'static str],
    /// The name of the setter called to the method table.
    pub set_function: &'static str,
    /// Represents a set of setters disabled by this setter.
    /// E.g., `set_setdelitem` have to disable `set_setitem` and `set_delitem`.
    pub skipped_setters: &'static [&'static str],
}

impl SlotSetter {
    const EMPTY_SETTERS: &'static [&'static str] = &[];
    const fn new(names: &'static [&'static str], set_function: &'static str) -> Self {
        SlotSetter {
            proto_names: names,
            set_function,
            skipped_setters: Self::EMPTY_SETTERS,
        }
    }
}

pub const OBJECT: Proto = Proto {
    name: "Object",
    slot_table: "pyo3::class::basic::PyObjectMethods",
    set_slot_table: "set_basic_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__str__"], "set_str"),
        SlotSetter::new(&["__repr__"], "set_repr"),
        SlotSetter::new(&["__hash__"], "set_hash"),
        SlotSetter::new(&["__getattr__"], "set_getattr"),
        SlotSetter::new(&["__richcmp__"], "set_richcompare"),
        SlotSetter {
            proto_names: &["__setattr__", "__delattr__"],
            set_function: "set_setdelattr",
            skipped_setters: &["set_setattr", "set_delattr"],
        },
        SlotSetter::new(&["__setattr__"], "set_setattr"),
        SlotSetter::new(&["__delattr__"], "set_delattr"),
        SlotSetter::new(&["__bool__"], "set_bool"),
    ],
};

pub const ASYNC: Proto = Proto {
    name: "Async",
    slot_table: "pyo3::ffi::PyAsyncMethods",
    set_slot_table: "set_async_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__await__"], "set_await"),
        SlotSetter::new(&["__aiter__"], "set_aiter"),
        SlotSetter::new(&["__anext__"], "set_anext"),
    ],
};

pub const BUFFER: Proto = Proto {
    name: "Buffer",
    slot_table: "pyo3::ffi::PyBufferProcs",
    set_slot_table: "set_buffer_methods",
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
    slot_setters: &[
        SlotSetter::new(&["bf_getbuffer"], "set_getbuffer"),
        SlotSetter::new(&["bf_releasebuffer"], "set_releasebuffer"),
    ],
};

pub const CONTEXT: Proto = Proto {
    name: "Context",
    slot_table: "",
    set_slot_table: "",
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
    slot_setters: &[],
};

pub const GC: Proto = Proto {
    name: "GC",
    slot_table: "pyo3::class::gc::PyGCMethods",
    set_slot_table: "set_gc_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__traverse__"], "set_traverse"),
        SlotSetter::new(&["__clear__"], "set_clear"),
    ],
};

pub const DESCR: Proto = Proto {
    name: "Descriptor",
    slot_table: "pyo3::class::descr::PyDescrMethods",
    set_slot_table: "set_descr_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__get__"], "set_descr_get"),
        SlotSetter::new(&["__set__"], "set_descr_set"),
    ],
};

pub const ITER: Proto = Proto {
    name: "Iter",
    slot_table: "pyo3::class::iter::PyIterMethods",
    set_slot_table: "set_iter_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__iter__"], "set_iter"),
        SlotSetter::new(&["__next__"], "set_iternext"),
    ],
};

pub const MAPPING: Proto = Proto {
    name: "Mapping",
    slot_table: "pyo3::ffi::PyMappingMethods",
    set_slot_table: "set_mapping_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__len__"], "set_length"),
        SlotSetter::new(&["__getitem__"], "set_getitem"),
        SlotSetter {
            proto_names: &["__setitem__", "__delitem__"],
            set_function: "set_setdelitem",
            skipped_setters: &["set_setitem", "set_delitem"],
        },
        SlotSetter::new(&["__setitem__"], "set_setitem"),
        SlotSetter::new(&["__delitem__"], "set_delitem"),
    ],
};

pub const SEQ: Proto = Proto {
    name: "Sequence",
    slot_table: "pyo3::ffi::PySequenceMethods",
    set_slot_table: "set_sequence_methods",
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
    slot_setters: &[
        SlotSetter::new(&["__len__"], "set_len"),
        SlotSetter::new(&["__concat__"], "set_concat"),
        SlotSetter::new(&["__repeat__"], "set_repeat"),
        SlotSetter::new(&["__getitem__"], "set_getitem"),
        SlotSetter {
            proto_names: &["__setitem__", "__delitem__"],
            set_function: "set_setdelitem",
            skipped_setters: &["set_setitem", "set_delitem"],
        },
        SlotSetter::new(&["__setitem__"], "set_setitem"),
        SlotSetter::new(&["__delitem__"], "set_delitem"),
        SlotSetter::new(&["__contains__"], "set_contains"),
        SlotSetter::new(&["__inplace_concat__"], "set_inplace_concat"),
        SlotSetter::new(&["__inplace_repeat__"], "set_inplace_repeat"),
    ],
};

pub const NUM: Proto = Proto {
    name: "Number",
    slot_table: "pyo3::ffi::PyNumberMethods",
    set_slot_table: "set_number_methods",
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
    slot_setters: &[
        SlotSetter {
            proto_names: &["__add__"],
            set_function: "set_add",
            skipped_setters: &["set_radd"],
        },
        SlotSetter::new(&["__radd__"], "set_radd"),
        SlotSetter {
            proto_names: &["__sub__"],
            set_function: "set_sub",
            skipped_setters: &["set_rsub"],
        },
        SlotSetter::new(&["__rsub__"], "set_rsub"),
        SlotSetter {
            proto_names: &["__mul__"],
            set_function: "set_mul",
            skipped_setters: &["set_rmul"],
        },
        SlotSetter::new(&["__rmul__"], "set_rmul"),
        SlotSetter::new(&["__mod__"], "set_mod"),
        SlotSetter {
            proto_names: &["__divmod__"],
            set_function: "set_divmod",
            skipped_setters: &["set_rdivmod"],
        },
        SlotSetter::new(&["__rdivmod__"], "set_rdivmod"),
        SlotSetter {
            proto_names: &["__pow__"],
            set_function: "set_pow",
            skipped_setters: &["set_rpow"],
        },
        SlotSetter::new(&["__rpow__"], "set_rpow"),
        SlotSetter::new(&["__neg__"], "set_neg"),
        SlotSetter::new(&["__pos__"], "set_pos"),
        SlotSetter::new(&["__abs__"], "set_abs"),
        SlotSetter::new(&["__invert__"], "set_invert"),
        SlotSetter::new(&["__rdivmod__"], "set_rdivmod"),
        SlotSetter {
            proto_names: &["__lshift__"],
            set_function: "set_lshift",
            skipped_setters: &["set_rlshift"],
        },
        SlotSetter::new(&["__rlshift__"], "set_rlshift"),
        SlotSetter {
            proto_names: &["__rshift__"],
            set_function: "set_rshift",
            skipped_setters: &["set_rrshift"],
        },
        SlotSetter::new(&["__rrshift__"], "set_rrshift"),
        SlotSetter {
            proto_names: &["__and__"],
            set_function: "set_and",
            skipped_setters: &["set_rand"],
        },
        SlotSetter::new(&["__rand__"], "set_rand"),
        SlotSetter {
            proto_names: &["__xor__"],
            set_function: "set_xor",
            skipped_setters: &["set_rxor"],
        },
        SlotSetter::new(&["__rxor__"], "set_rxor"),
        SlotSetter {
            proto_names: &["__or__"],
            set_function: "set_or",
            skipped_setters: &["set_ror"],
        },
        SlotSetter::new(&["__ror__"], "set_ror"),
        SlotSetter::new(&["__int__"], "set_int"),
        SlotSetter::new(&["__float__"], "set_float"),
        SlotSetter::new(&["__iadd__"], "set_iadd"),
        SlotSetter::new(&["__isub__"], "set_isub"),
        SlotSetter::new(&["__imul__"], "set_imul"),
        SlotSetter::new(&["__imod__"], "set_imod"),
        SlotSetter::new(&["__ipow__"], "set_ipow"),
        SlotSetter::new(&["__ilshift__"], "set_ilshift"),
        SlotSetter::new(&["__irshift__"], "set_irshift"),
        SlotSetter::new(&["__iand__"], "set_iand"),
        SlotSetter::new(&["__ixor__"], "set_ixor"),
        SlotSetter::new(&["__ior__"], "set_ior"),
        SlotSetter {
            proto_names: &["__floordiv__"],
            set_function: "set_floordiv",
            skipped_setters: &["set_rfloordiv"],
        },
        SlotSetter::new(&["__rfloordiv__"], "set_rfloordiv"),
        SlotSetter {
            proto_names: &["__truediv__"],
            set_function: "set_truediv",
            skipped_setters: &["set_rtruediv"],
        },
        SlotSetter::new(&["__rtruediv__"], "set_rtruediv"),
        SlotSetter::new(&["__ifloordiv__"], "set_ifloordiv"),
        SlotSetter::new(&["__itruediv__"], "set_itruediv"),
        SlotSetter::new(&["__index__"], "set_index"),
        SlotSetter {
            proto_names: &["__matmul__"],
            set_function: "set_matmul",
            skipped_setters: &["set_rmatmul"],
        },
        SlotSetter::new(&["__rmatmul__"], "set_rmatmul"),
        SlotSetter::new(&["__imatmul__"], "set_imatmul"),
    ],
};
