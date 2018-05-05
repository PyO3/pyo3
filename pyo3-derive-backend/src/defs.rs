// Copyright (c) 2017-present PyO3 Project and Contributors
use func::MethodProto;

pub struct Proto {
    pub name: &'static str,
    pub methods: &'static [MethodProto],
    pub py_methods: &'static [PyMethod],
}

pub struct PyMethod {
    pub name: &'static str,
    pub proto: &'static str,
}

pub const OBJECT: Proto = Proto {
    name: "Object",
    methods: &[
        MethodProto::Binary {
            name: "__getattr__",
            arg: "Name",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectGetAttrProtocol"},
        MethodProto::Ternary {
            name: "__setattr__",
            arg1: "Name",
            arg2: "Value",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectSetAttrProtocol"},
        MethodProto::Binary {
            name: "__delattr__",
            arg: "Name",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectDelAttrProtocol"},
        MethodProto::Unary {
            name: "__str__",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectStrProtocol"},
        MethodProto::Unary {
            name: "__repr__",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectReprProtocol"},
        MethodProto::Binary {
            name: "__format__",
            arg: "Format",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectFormatProtocol"},
        MethodProto::Unary {
            name: "__hash__",
            pyres: false,
            proto: "::pyo3::class::basic::PyObjectHashProtocol"},
        MethodProto::Unary {
            name: "__bytes__",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectBytesProtocol"},
        MethodProto::Unary {
            name: "__unicode__",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectUnicodeProtocol"},
        MethodProto::Unary {
            name: "__bool__",
            pyres: false,
            proto: "::pyo3::class::basic::PyObjectBoolProtocol"},
        MethodProto::Binary {
            name: "__richcmp__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::basic::PyObjectRichcmpProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__format__",
            proto: "::pyo3::class::basic::PyObjectFormatProtocolImpl",
        },
        PyMethod {
            name: "__bytes__",
            proto: "::pyo3::class::basic::PyObjectBytesProtocolImpl",
        },
        PyMethod {
            name: "__unicode__",
            proto: "::pyo3::class::basic::PyObjectUnicodeProtocolImpl",
        },
    ]
};


pub const ASYNC: Proto = Proto {
    name: "Async",
    methods: &[
        MethodProto::Unary {
            name: "__await__",
            pyres: true,
            proto: "::pyo3::class::async::PyAsyncAwaitProtocol"},
        MethodProto::Unary{
            name: "__aiter__",
            pyres: true,
            proto: "::pyo3::class::async::PyAsyncAiterProtocol"},
        MethodProto::Unary{
            name: "__anext__",
            pyres: true,
            proto: "::pyo3::class::async::PyAsyncAnextProtocol"},
        MethodProto::Unary{
            name: "__aenter__",
            pyres: true,
            proto: "::pyo3::class::async::PyAsyncAenterProtocol"},
        MethodProto::Quaternary {
            name: "__aexit__",
            arg1: "ExcType", arg2: "ExcValue", arg3: "Traceback",
            proto: "::pyo3::class::async::PyAsyncAexitProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__aenter__",
            proto: "::pyo3::class::async::PyAsyncAenterProtocolImpl",
        },
        PyMethod {
            name: "__aexit__",
            proto: "::pyo3::class::async::PyAsyncAexitProtocolImpl",
        },
    ],
};

pub const BUFFER: Proto = Proto {
    name: "Buffer",
    methods: &[
        MethodProto::Unary{
            name: "bf_getbuffer",
            pyres: false,
            proto: "::pyo3::class::buffer::PyBufferGetBufferProtocol"},
        MethodProto::Unary{
            name: "bf_releasebuffer",
            pyres: false,
            proto: "::pyo3::class::buffer::PyBufferReleaseBufferProtocol"},
    ],
    py_methods: &[],
};

pub const CONTEXT: Proto = Proto {
    name: "Context",
    methods: &[
        MethodProto::Unary{
            name: "__enter__",
            pyres: true,
            proto: "::pyo3::class::context::PyContextEnterProtocol"},
        MethodProto::Quaternary {
            name: "__exit__",
            arg1: "ExcType", arg2: "ExcValue", arg3: "Traceback",
            proto: "::pyo3::class::context::PyContextExitProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__enter__",
            proto: "_pyo3::class::context::PyContextEnterProtocolImpl",
        },
        PyMethod {
            name: "__exit__",
            proto: "_pyo3::class::context::PyContextExitProtocolImpl",
        },
    ],
};

pub const GC: Proto = Proto {
    name: "GC",
    methods: &[
        MethodProto::Free{
            name: "__traverse__",
            proto: "::pyo3::class::gc::PyGCTraverseProtocol"},
        MethodProto::Free{
            name: "__clear__",
            proto: "::pyo3::class::gc::PyGCClearProtocol"},
    ],
    py_methods: &[],
};

pub const DESCR: Proto = Proto {
    name: "Descriptor",
    methods: &[
        MethodProto::Ternary {
            name: "__get__",
            arg1: "Inst",
            arg2: "Owner",
            pyres: true,
            proto: "::pyo3::class::descr::PyDescrGetProtocol"},
        MethodProto::Ternary {
            name: "__set__",
            arg1: "Inst",
            arg2: "Value",
            pyres: true,
            proto: "::pyo3::class::descr::PyDescrSetProtocol"},
        MethodProto::Binary {
            name: "__det__",
            arg: "Inst",
            pyres: false,
            proto: "::pyo3::class::descr::PyDescrDelProtocol"},
        MethodProto::Binary {
            name: "__set_name__",
            arg: "Inst",
            pyres: false,
            proto: "::pyo3::class::descr::PyDescrSetNameProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__del__",
            proto: "_pyo3::class::context::PyDescrDelProtocolImpl",
        },
        PyMethod {
            name: "__set_name__",
            proto: "_pyo3::class::context::PyDescrNameProtocolImpl",
        },
    ],
};

pub const ITER: Proto = Proto {
    name: "Iter",
    py_methods: &[],
    methods: &[
        MethodProto::Unary{
            name: "__iter__",
            pyres: true,
            proto: "::pyo3::class::iter::PyIterIterProtocol"},
        MethodProto::Unary{
            name: "__next__",
            pyres: true,
            proto: "::pyo3::class::iter::PyIterNextProtocol"},
    ],
};


pub const MAPPING: Proto = Proto {
    name: "Mapping",
    methods: &[
        MethodProto::Unary{
            name: "__len__",
            pyres: false,
            proto: "::pyo3::class::mapping::PyMappingLenProtocol"},
        MethodProto::Binary{
            name: "__getitem__",
            arg: "Key",
            pyres: true,
            proto: "::pyo3::class::mapping::PyMappingGetItemProtocol"},
        MethodProto::Ternary{
            name: "__setitem__",
            arg1: "Key",
            arg2: "Value",
            pyres: false,
            proto: "::pyo3::class::mapping::PyMappingSetItemProtocol"},
        MethodProto::Binary{
            name: "__delitem__",
            arg: "Key",
            pyres: false,
            proto: "::pyo3::class::mapping::PyMappingDelItemProtocol"},
        MethodProto::Binary{
            name: "__contains__",
            arg: "Value",
            pyres: false,
            proto: "::pyo3::class::mapping::PyMappingContainsProtocol"},
        MethodProto::Unary{
            name: "__reversed__",
            pyres: true,
            proto: "::pyo3::class::mapping::PyMappingReversedProtocol"},
        MethodProto::Unary{
            name: "__iter__",
            pyres: true,
            proto: "::pyo3::class::mapping::PyMappingIterProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__iter__",
            proto: "::pyo3::class::mapping::PyMappingIterProtocolImpl",
        },
        PyMethod {
            name: "__contains__",
            proto: "::pyo3::class::mapping::PyMappingContainsProtocolImpl",
        },
        PyMethod {
            name: "__reversed__",
            proto: "::pyo3::class::mapping::PyMappingReversedProtocolImpl",
        },
    ],
};

pub const SEQ: Proto = Proto {
    name: "Sequence",
    methods: &[
        MethodProto::Unary{
            name: "__len__",
            pyres: false,
            proto: "pyo3::class::sequence::PySequenceLenProtocol"},
        MethodProto::Unary{
            name: "__getitem__",
            pyres: true,
            proto: "pyo3::class::sequence::PySequenceGetItemProtocol"},
        MethodProto::Binary{
            name: "__setitem__",
            arg: "Value",
            pyres: false,
            proto: "pyo3::class::sequence::PyMappingSetItemProtocol"},
        MethodProto::Binary{
            name: "__delitem__",
            arg: "Key",
            pyres: false,
            proto: "pyo3::class::mapping::PyMappingDelItemProtocol"},
        MethodProto::Binary{
            name: "__contains__",
            arg: "Item",
            pyres: false,
            proto: "pyo3::class::sequence::PySequenceContainsProtocol"},
        MethodProto::Binary{
            name: "__concat__",
            arg: "Other",
            pyres: true,
            proto: "pyo3::class::sequence::PySequenceConcatProtocol"},
        MethodProto::Unary{
            name: "__repeat__",
            pyres: true,
            proto: "pyo3::class::sequence::PySequenceRepeatProtocol"},
        MethodProto::Binary{
            name: "__inplace_concat__",
            arg: "Other",
            pyres: true,
            proto: "pyo3::class::sequence::PySequenceInplaceConcatProtocol"},
        MethodProto::Unary{
            name: "__inplace_repeat__",
            pyres: true,
            proto: "pyo3::class::sequence::PySequenceInplaceRepeatProtocol"},
    ],
    py_methods: &[],
};

pub const NUM: Proto = Proto {
    name: "Number",
    methods: &[
        MethodProto::BinaryS {
            name: "__add__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberAddProtocol"},
        MethodProto::BinaryS {
            name: "__sub__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberSubProtocol"},
        MethodProto::BinaryS {
            name: "__mul__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberMulProtocol"},
        MethodProto::BinaryS {
            name: "__matmul__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberMatmulProtocol"},
        MethodProto::BinaryS {
            name: "__truediv__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberTruedivProtocol"},
        MethodProto::BinaryS {
            name: "__floordiv__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberFloordivProtocol"},
        MethodProto::BinaryS {
            name: "__mod__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberModProtocol"},
        MethodProto::BinaryS {
            name: "__divmod__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberDivmodProtocol"},
        MethodProto::TernaryS {
            name: "__pow__",
            arg1: "Left",
            arg2: "Right",
            arg3: "Modulo",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberPowProtocol"},
        MethodProto::BinaryS {
            name: "__lshift__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberLShiftProtocol"},
        MethodProto::BinaryS {
            name: "__rshift__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRShiftProtocol"},
        MethodProto::BinaryS {
            name: "__and__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberAndProtocol"},
        MethodProto::BinaryS {
            name: "__xor__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberXorProtocol"},
        MethodProto::BinaryS {
            name: "__or__",
            arg1: "Left",
            arg2: "Right",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberOrProtocol"},

        MethodProto::Binary {
            name: "__radd__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRAddProtocol"},
        MethodProto::Binary {
            name: "__rsub__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRSubProtocol"},
        MethodProto::Binary {
            name: "__rmul__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRMulProtocol"},
        MethodProto::Binary {
            name: "__rmatmul__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRMatmulProtocol"},
        MethodProto::Binary {
            name: "__rtruediv__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRTruedivProtocol"},
        MethodProto::Binary {
            name: "__rfloordiv__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRFloordivProtocol"},
        MethodProto::Binary {
            name: "__rmod__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRModProtocol"},
        MethodProto::Binary {
            name: "__rdivmod__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRDivmodProtocol"},
        MethodProto::Ternary {
            name: "__rpow__",
            arg1: "Other",
            arg2: "Modulo",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRPowProtocol"},
        MethodProto::Binary {
            name: "__rlshift__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRLShiftProtocol"},
        MethodProto::Binary {
            name: "__rrshift__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRRShiftProtocol"},
        MethodProto::Binary {
            name: "__rand__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRAndProtocol"},
        MethodProto::Binary {
            name: "__rxor__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRXorProtocol"},
        MethodProto::Binary {
            name: "__ror__",
            arg: "Other",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberROrProtocol"},

        MethodProto::Binary {
            name: "__iadd__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIAddProtocol"},
        MethodProto::Binary {
            name: "__isub__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberISubProtocol"},
        MethodProto::Binary {
            name: "__imul__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIMulProtocol"},
        MethodProto::Binary {
            name: "__imatmul__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIMatmulProtocol"},
        MethodProto::Binary {
            name: "__itruediv__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberITruedivProtocol"},
        MethodProto::Binary {
            name: "__ifloordiv__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIFloordivProtocol"},
        MethodProto::Binary {
            name: "__imod__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIModProtocol"},
        MethodProto::Ternary {
            name: "__ipow__",
            arg1: "Other",
            arg2: "Modulo",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIPowProtocol"},
        MethodProto::Binary {
            name: "__ilshift__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberILShiftProtocol"},
        MethodProto::Binary {
            name: "__irshift__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIRShiftProtocol"},
        MethodProto::Binary {
            name: "__iand__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIAndProtocol"},
        MethodProto::Binary {
            name: "__ixor__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIXorProtocol"},
        MethodProto::Binary {
            name: "__ior__",
            arg: "Other",
            pyres: false,
            proto: "::pyo3::class::number::PyNumberIOrProtocol"},

        MethodProto::Unary {
            name: "__neg__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberNegProtocol"},
        MethodProto::Unary {
            name: "__pos__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberPosProtocol"},
        MethodProto::Unary {
            name: "__abs__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberAbsProtocol"},
        MethodProto::Unary {
            name: "__invert__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberInvertProtocol"},
        MethodProto::Unary {
            name: "__complex__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberComplexProtocol"},
        MethodProto::Unary {
            name: "__int__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberIntProtocol"},
        MethodProto::Unary {
            name: "__float__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberFloatProtocol"},
        MethodProto::Unary {
            name: "__round__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberRoundProtocol"},
        MethodProto::Unary {
            name: "__index__",
            pyres: true,
            proto: "::pyo3::class::number::PyNumberIndexProtocol"},
    ],
    py_methods: &[
        PyMethod {
            name: "__radd__",
            proto: "_pyo3::class::number::PyNumberRAddProtocolImpl"},
        PyMethod {
            name: "__rsub__",
            proto: "_pyo3::class::number::PyNumberRSubProtocolImpl"},
        PyMethod {
            name: "__rmul__",
            proto: "_pyo3::class::number::PyNumberRMulProtocolImpl"},
        PyMethod {
            name: "__rmatmul__",
            proto: "_pyo3::class::number::PyNumberRMatmulProtocolImpl"},
        PyMethod {
            name: "__rtruediv__",
            proto: "_pyo3::class::number::PyNumberRTruedivProtocolImpl"},
        PyMethod {
            name: "__rfloordiv__",
            proto: "_pyo3::class::number::PyNumberRFloordivProtocolImpl"},
        PyMethod {
            name: "__rmod__",
            proto: "_pyo3::class::number::PyNumberRModProtocolImpl"},
        PyMethod {
            name: "__rdivmod__",
            proto: "_pyo3::class::number::PyNumberRDivmodProtocolImpl"},
        PyMethod {
            name: "__rpow__",
            proto: "_pyo3::class::number::PyNumberRPowProtocolImpl"},
        PyMethod {
            name: "__rlshift__",
            proto: "_pyo3::class::number::PyNumberRLShiftProtocolImpl"},
        PyMethod {
            name: "__rrshift__",
            proto: "_pyo3::class::number::PyNumberRRShiftProtocolImpl"},
        PyMethod {
            name: "__rand__",
            proto: "_pyo3::class::number::PyNumberRAndProtocolImpl"},
        PyMethod {
            name: "__rxor__",
            proto: "_pyo3::class::number::PyNumberRXorProtocolImpl"},
        PyMethod {
            name: "__ror__",
            proto: "_pyo3::class::number::PyNumberROrProtocolImpl"},
        PyMethod {
            name: "__complex__",
            proto: "_pyo3::class::number::PyNumberComplexProtocolImpl"},
        PyMethod {
            name: "__round__",
            proto: "_pyo3::class::number::PyNumberRoundProtocolImpl"},
    ]
};
