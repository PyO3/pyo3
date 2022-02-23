// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::proto_method::MethodProto;
use proc_macro2::Span;
use std::collections::HashSet;

/// Predicates for `#[pyproto]`.
pub struct Proto {
    /// The name of this protocol. E.g., Iter.
    pub name: &'static str,
    /// The path to the module which contains this proto implementation.
    module: &'static str,
    /// Trait which stores the slots
    /// Trait method which accesses the slots.
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

    // Returns the hard-coded module as a path
    #[inline]
    pub(crate) fn module(&self) -> syn::Path {
        syn::parse_str(self.module).expect("module def not valid path")
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

    pub(crate) fn items_trait(&self) -> syn::Ident {
        syn::Ident::new(&format!("Py{}ProtocolItems", self.name), Span::call_site())
    }

    pub(crate) fn items_trait_items(&self) -> syn::Ident {
        syn::Ident::new(
            &format!("{}_protocol_items", self.name.to_ascii_lowercase()),
            Span::call_site(),
        )
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
    module: "::pyo3::class::basic",
    methods: &[
        MethodProto::new("__getattr__", "PyObjectGetAttrProtocol")
            .args(&["Name"])
            .has_self(),
        MethodProto::new("__setattr__", "PyObjectSetAttrProtocol")
            .args(&["Name", "Value"])
            .has_self(),
        MethodProto::new("__delattr__", "PyObjectDelAttrProtocol")
            .args(&["Name"])
            .has_self(),
        MethodProto::new("__str__", "PyObjectStrProtocol").has_self(),
        MethodProto::new("__repr__", "PyObjectReprProtocol").has_self(),
        MethodProto::new("__hash__", "PyObjectHashProtocol").has_self(),
        MethodProto::new("__richcmp__", "PyObjectRichcmpProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__bool__", "PyObjectBoolProtocol").has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__str__"], "Py_tp_str", "str"),
        SlotDef::new(&["__repr__"], "Py_tp_repr", "repr"),
        SlotDef::new(&["__hash__"], "Py_tp_hash", "hash"),
        SlotDef::new(&["__getattr__"], "Py_tp_getattro", "getattr"),
        SlotDef::new(&["__richcmp__"], "Py_tp_richcompare", "richcmp"),
        SlotDef::new(
            &["__setattr__", "__delattr__"],
            "Py_tp_setattro",
            "setdelattr",
        ),
        SlotDef::new(&["__setattr__"], "Py_tp_setattro", "setattr"),
        SlotDef::new(&["__delattr__"], "Py_tp_setattro", "delattr"),
        SlotDef::new(&["__bool__"], "Py_nb_bool", "bool"),
    ],
};

pub const ASYNC: Proto = Proto {
    name: "Async",
    module: "::pyo3::class::pyasync",
    methods: &[
        MethodProto::new("__await__", "PyAsyncAwaitProtocol").args(&["Receiver"]),
        MethodProto::new("__aiter__", "PyAsyncAiterProtocol").args(&["Receiver"]),
        MethodProto::new("__anext__", "PyAsyncAnextProtocol").args(&["Receiver"]),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__await__"], "Py_am_await", "await_"),
        SlotDef::new(&["__aiter__"], "Py_am_aiter", "aiter"),
        SlotDef::new(&["__anext__"], "Py_am_anext", "anext"),
    ],
};

pub const BUFFER: Proto = Proto {
    name: "Buffer",
    module: "::pyo3::class::buffer",
    methods: &[
        MethodProto::new("bf_getbuffer", "PyBufferGetBufferProtocol").has_self(),
        MethodProto::new("bf_releasebuffer", "PyBufferReleaseBufferProtocol").has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["bf_getbuffer"], "Py_bf_getbuffer", "getbuffer"),
        SlotDef::new(
            &["bf_releasebuffer"],
            "Py_bf_releasebuffer",
            "releasebuffer",
        ),
    ],
};

pub const GC: Proto = Proto {
    name: "GC",
    module: "::pyo3::class::gc",
    methods: &[
        MethodProto::new("__traverse__", "PyGCTraverseProtocol")
            .has_self()
            .no_result(),
        MethodProto::new("__clear__", "PyGCClearProtocol")
            .has_self()
            .no_result(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__traverse__"], "Py_tp_traverse", "traverse"),
        SlotDef::new(&["__clear__"], "Py_tp_clear", "clear"),
    ],
};

pub const DESCR: Proto = Proto {
    name: "Descr",
    module: "::pyo3::class::descr",
    methods: &[
        MethodProto::new("__get__", "PyDescrGetProtocol").args(&["Receiver", "Inst", "Owner"]),
        MethodProto::new("__set__", "PyDescrSetProtocol").args(&["Receiver", "Inst", "Value"]),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__get__"], "Py_tp_descr_get", "descr_get"),
        SlotDef::new(&["__set__"], "Py_tp_descr_set", "descr_set"),
    ],
};

pub const ITER: Proto = Proto {
    name: "Iter",
    module: "::pyo3::class::iter",
    py_methods: &[],
    methods: &[
        MethodProto::new("__iter__", "PyIterIterProtocol").args(&["Receiver"]),
        MethodProto::new("__next__", "PyIterNextProtocol").args(&["Receiver"]),
    ],
    slot_defs: &[
        SlotDef::new(&["__iter__"], "Py_tp_iter", "iter"),
        SlotDef::new(&["__next__"], "Py_tp_iternext", "iternext"),
    ],
};

pub const MAPPING: Proto = Proto {
    name: "Mapping",
    module: "::pyo3::class::mapping",
    methods: &[
        MethodProto::new("__len__", "PyMappingLenProtocol").has_self(),
        MethodProto::new("__getitem__", "PyMappingGetItemProtocol")
            .args(&["Key"])
            .has_self(),
        MethodProto::new("__setitem__", "PyMappingSetItemProtocol")
            .args(&["Key", "Value"])
            .has_self(),
        MethodProto::new("__delitem__", "PyMappingDelItemProtocol")
            .args(&["Key"])
            .has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__len__"], "Py_mp_length", "len"),
        SlotDef::new(&["__getitem__"], "Py_mp_subscript", "getitem"),
        SlotDef::new(
            &["__setitem__", "__delitem__"],
            "Py_mp_ass_subscript",
            "setdelitem",
        ),
        SlotDef::new(&["__setitem__"], "Py_mp_ass_subscript", "setitem"),
        SlotDef::new(&["__delitem__"], "Py_mp_ass_subscript", "delitem"),
    ],
};

pub const SEQ: Proto = Proto {
    name: "Sequence",
    module: "::pyo3::class::sequence",
    methods: &[
        MethodProto::new("__len__", "PySequenceLenProtocol").has_self(),
        MethodProto::new("__getitem__", "PySequenceGetItemProtocol")
            .args(&["Index"])
            .has_self(),
        MethodProto::new("__setitem__", "PySequenceSetItemProtocol")
            .args(&["Index", "Value"])
            .has_self(),
        MethodProto::new("__delitem__", "PySequenceDelItemProtocol")
            .args(&["Index"])
            .has_self(),
        MethodProto::new("__contains__", "PySequenceContainsProtocol")
            .args(&["Item"])
            .has_self(),
        MethodProto::new("__concat__", "PySequenceConcatProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__repeat__", "PySequenceRepeatProtocol")
            .args(&["Index"])
            .has_self(),
        MethodProto::new("__inplace_concat__", "PySequenceInplaceConcatProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__inplace_repeat__", "PySequenceInplaceRepeatProtocol")
            .args(&["Index"])
            .has_self(),
    ],
    py_methods: &[],
    slot_defs: &[
        SlotDef::new(&["__len__"], "Py_sq_length", "len"),
        SlotDef::new(&["__concat__"], "Py_sq_concat", "concat"),
        SlotDef::new(&["__repeat__"], "Py_sq_repeat", "repeat"),
        SlotDef::new(&["__getitem__"], "Py_sq_item", "getitem"),
        SlotDef::new(
            &["__setitem__", "__delitem__"],
            "Py_sq_ass_item",
            "setdelitem",
        ),
        SlotDef::new(&["__setitem__"], "Py_sq_ass_item", "setitem"),
        SlotDef::new(&["__delitem__"], "Py_sq_ass_item", "delitem"),
        SlotDef::new(&["__contains__"], "Py_sq_contains", "contains"),
        SlotDef::new(
            &["__inplace_concat__"],
            "Py_sq_inplace_concat",
            "inplace_concat",
        ),
        SlotDef::new(
            &["__inplace_repeat__"],
            "Py_sq_inplace_repeat",
            "inplace_repeat",
        ),
    ],
};

pub const NUM: Proto = Proto {
    name: "Number",
    module: "::pyo3::class::number",
    methods: &[
        MethodProto::new("__add__", "PyNumberAddProtocol").args(&["Left", "Right"]),
        MethodProto::new("__sub__", "PyNumberSubProtocol").args(&["Left", "Right"]),
        MethodProto::new("__mul__", "PyNumberMulProtocol").args(&["Left", "Right"]),
        MethodProto::new("__matmul__", "PyNumberMatmulProtocol").args(&["Left", "Right"]),
        MethodProto::new("__truediv__", "PyNumberTruedivProtocol").args(&["Left", "Right"]),
        MethodProto::new("__floordiv__", "PyNumberFloordivProtocol").args(&["Left", "Right"]),
        MethodProto::new("__mod__", "PyNumberModProtocol").args(&["Left", "Right"]),
        MethodProto::new("__divmod__", "PyNumberDivmodProtocol").args(&["Left", "Right"]),
        MethodProto::new("__pow__", "PyNumberPowProtocol").args(&["Left", "Right", "Modulo"]),
        MethodProto::new("__lshift__", "PyNumberLShiftProtocol").args(&["Left", "Right"]),
        MethodProto::new("__rshift__", "PyNumberRShiftProtocol").args(&["Left", "Right"]),
        MethodProto::new("__and__", "PyNumberAndProtocol").args(&["Left", "Right"]),
        MethodProto::new("__xor__", "PyNumberXorProtocol").args(&["Left", "Right"]),
        MethodProto::new("__or__", "PyNumberOrProtocol").args(&["Left", "Right"]),
        MethodProto::new("__radd__", "PyNumberRAddProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rsub__", "PyNumberRSubProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rmul__", "PyNumberRMulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rmatmul__", "PyNumberRMatmulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rtruediv__", "PyNumberRTruedivProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rfloordiv__", "PyNumberRFloordivProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rmod__", "PyNumberRModProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rdivmod__", "PyNumberRDivmodProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rpow__", "PyNumberRPowProtocol")
            .args(&["Other", "Modulo"])
            .has_self(),
        MethodProto::new("__rlshift__", "PyNumberRLShiftProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rrshift__", "PyNumberRRShiftProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rand__", "PyNumberRAndProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__rxor__", "PyNumberRXorProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ror__", "PyNumberROrProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__iadd__", "PyNumberIAddProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__isub__", "PyNumberISubProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__imul__", "PyNumberIMulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__imatmul__", "PyNumberIMatmulProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__itruediv__", "PyNumberITruedivProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ifloordiv__", "PyNumberIFloordivProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__imod__", "PyNumberIModProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ipow__", "PyNumberIPowProtocol")
            .args(&["Other", "Modulo"])
            .has_self(),
        MethodProto::new("__ilshift__", "PyNumberILShiftProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__irshift__", "PyNumberIRShiftProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__iand__", "PyNumberIAndProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ixor__", "PyNumberIXorProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__ior__", "PyNumberIOrProtocol")
            .args(&["Other"])
            .has_self(),
        MethodProto::new("__neg__", "PyNumberNegProtocol").has_self(),
        MethodProto::new("__pos__", "PyNumberPosProtocol").has_self(),
        MethodProto::new("__abs__", "PyNumberAbsProtocol").has_self(),
        MethodProto::new("__invert__", "PyNumberInvertProtocol").has_self(),
        MethodProto::new("__int__", "PyNumberIntProtocol").has_self(),
        MethodProto::new("__float__", "PyNumberFloatProtocol").has_self(),
        MethodProto::new("__index__", "PyNumberIndexProtocol").has_self(),
    ],
    py_methods: &[
        PyMethod::coexist("__radd__", "PyNumberRAddProtocolImpl"),
        PyMethod::coexist("__rsub__", "PyNumberRSubProtocolImpl"),
        PyMethod::coexist("__rmul__", "PyNumberRMulProtocolImpl"),
        PyMethod::coexist("__rmatmul__", "PyNumberRMatmulProtocolImpl"),
        PyMethod::coexist("__rtruediv__", "PyNumberRTruedivProtocolImpl"),
        PyMethod::coexist("__rfloordiv__", "PyNumberRFloordivProtocolImpl"),
        PyMethod::coexist("__rmod__", "PyNumberRModProtocolImpl"),
        PyMethod::coexist("__rdivmod__", "PyNumberRDivmodProtocolImpl"),
        PyMethod::coexist("__rpow__", "PyNumberRPowProtocolImpl"),
        PyMethod::coexist("__rlshift__", "PyNumberRLShiftProtocolImpl"),
        PyMethod::coexist("__rrshift__", "PyNumberRRShiftProtocolImpl"),
        PyMethod::coexist("__rand__", "PyNumberRAndProtocolImpl"),
        PyMethod::coexist("__rxor__", "PyNumberRXorProtocolImpl"),
        PyMethod::coexist("__ror__", "PyNumberROrProtocolImpl"),
    ],
    slot_defs: &[
        SlotDef::new(&["__add__", "__radd__"], "Py_nb_add", "add_radd"),
        SlotDef::new(&["__add__"], "Py_nb_add", "add"),
        SlotDef::new(&["__radd__"], "Py_nb_add", "radd"),
        SlotDef::new(&["__sub__", "__rsub__"], "Py_nb_subtract", "sub_rsub"),
        SlotDef::new(&["__sub__"], "Py_nb_subtract", "sub"),
        SlotDef::new(&["__rsub__"], "Py_nb_subtract", "rsub"),
        SlotDef::new(&["__mul__", "__rmul__"], "Py_nb_multiply", "mul_rmul"),
        SlotDef::new(&["__mul__"], "Py_nb_multiply", "mul"),
        SlotDef::new(&["__rmul__"], "Py_nb_multiply", "rmul"),
        SlotDef::new(&["__mod__", "__rmod__"], "Py_nb_remainder", "mod_rmod"),
        SlotDef::new(&["__mod__"], "Py_nb_remainder", "mod_"),
        SlotDef::new(&["__rmod__"], "Py_nb_remainder", "rmod"),
        SlotDef::new(
            &["__divmod__", "__rdivmod__"],
            "Py_nb_divmod",
            "divmod_rdivmod",
        ),
        SlotDef::new(&["__divmod__"], "Py_nb_divmod", "divmod"),
        SlotDef::new(&["__rdivmod__"], "Py_nb_divmod", "rdivmod"),
        SlotDef::new(&["__pow__", "__rpow__"], "Py_nb_power", "pow_rpow"),
        SlotDef::new(&["__pow__"], "Py_nb_power", "pow"),
        SlotDef::new(&["__rpow__"], "Py_nb_power", "rpow"),
        SlotDef::new(&["__neg__"], "Py_nb_negative", "neg"),
        SlotDef::new(&["__pos__"], "Py_nb_positive", "pos"),
        SlotDef::new(&["__abs__"], "Py_nb_absolute", "abs"),
        SlotDef::new(&["__invert__"], "Py_nb_invert", "invert"),
        SlotDef::new(
            &["__lshift__", "__rlshift__"],
            "Py_nb_lshift",
            "lshift_rlshift",
        ),
        SlotDef::new(&["__lshift__"], "Py_nb_lshift", "lshift"),
        SlotDef::new(&["__rlshift__"], "Py_nb_lshift", "rlshift"),
        SlotDef::new(
            &["__rshift__", "__rrshift__"],
            "Py_nb_rshift",
            "rshift_rrshift",
        ),
        SlotDef::new(&["__rshift__"], "Py_nb_rshift", "rshift"),
        SlotDef::new(&["__rrshift__"], "Py_nb_rshift", "rrshift"),
        SlotDef::new(&["__and__", "__rand__"], "Py_nb_and", "and_rand"),
        SlotDef::new(&["__and__"], "Py_nb_and", "and"),
        SlotDef::new(&["__rand__"], "Py_nb_and", "rand"),
        SlotDef::new(&["__xor__", "__rxor__"], "Py_nb_xor", "xor_rxor"),
        SlotDef::new(&["__xor__"], "Py_nb_xor", "xor"),
        SlotDef::new(&["__rxor__"], "Py_nb_xor", "rxor"),
        SlotDef::new(&["__or__", "__ror__"], "Py_nb_or", "or_ror"),
        SlotDef::new(&["__or__"], "Py_nb_or", "or"),
        SlotDef::new(&["__ror__"], "Py_nb_or", "ror"),
        SlotDef::new(&["__int__"], "Py_nb_int", "int"),
        SlotDef::new(&["__float__"], "Py_nb_float", "float"),
        SlotDef::new(&["__iadd__"], "Py_nb_inplace_add", "iadd"),
        SlotDef::new(&["__isub__"], "Py_nb_inplace_subtract", "isub"),
        SlotDef::new(&["__imul__"], "Py_nb_inplace_multiply", "imul"),
        SlotDef::new(&["__imod__"], "Py_nb_inplace_remainder", "imod"),
        SlotDef::new(&["__ipow__"], "Py_nb_inplace_power", "ipow"),
        SlotDef::new(&["__ilshift__"], "Py_nb_inplace_lshift", "ilshift"),
        SlotDef::new(&["__irshift__"], "Py_nb_inplace_rshift", "irshift"),
        SlotDef::new(&["__iand__"], "Py_nb_inplace_and", "iand"),
        SlotDef::new(&["__ixor__"], "Py_nb_inplace_xor", "ixor"),
        SlotDef::new(&["__ior__"], "Py_nb_inplace_or", "ior"),
        SlotDef::new(
            &["__floordiv__", "__rfloordiv__"],
            "Py_nb_floor_divide",
            "floordiv_rfloordiv",
        ),
        SlotDef::new(&["__floordiv__"], "Py_nb_floor_divide", "floordiv"),
        SlotDef::new(&["__rfloordiv__"], "Py_nb_floor_divide", "rfloordiv"),
        SlotDef::new(
            &["__truediv__", "__rtruediv__"],
            "Py_nb_true_divide",
            "truediv_rtruediv",
        ),
        SlotDef::new(&["__truediv__"], "Py_nb_true_divide", "truediv"),
        SlotDef::new(&["__rtruediv__"], "Py_nb_true_divide", "rtruediv"),
        SlotDef::new(
            &["__ifloordiv__"],
            "Py_nb_inplace_floor_divide",
            "ifloordiv",
        ),
        SlotDef::new(&["__itruediv__"], "Py_nb_inplace_true_divide", "itruediv"),
        SlotDef::new(&["__index__"], "Py_nb_index", "index"),
        SlotDef::new(
            &["__matmul__", "__rmatmul__"],
            "Py_nb_matrix_multiply",
            "matmul_rmatmul",
        ),
        SlotDef::new(&["__matmul__"], "Py_nb_matrix_multiply", "matmul"),
        SlotDef::new(&["__rmatmul__"], "Py_nb_matrix_multiply", "rmatmul"),
        SlotDef::new(&["__imatmul__"], "Py_nb_inplace_matrix_multiply", "imatmul"),
    ],
};
