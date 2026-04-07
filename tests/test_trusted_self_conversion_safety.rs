#![cfg(feature = "macros")]
//! Safety tests for `SelfConversionPolicy::Trusted`.
//!
//! For every pyo3-generated wrapper that uses `SelfConversionPolicy::Trusted`
//! (i.e. the Rust code skips the `isinstance` check), CPython's own dispatch
//! machinery ensures the receiver is an instance of the correct type *before*
//! our C function is ever called.  The tests below verify that invariant: for
//! each slot category, passing a wrong-type receiver from Python raises
//! `TypeError`, proving it is safe to skip the redundant Rust-side check.

use pyo3::prelude::*;

mod test_utils;

// ---------------------------------------------------------------------------
// tp_str / tp_repr / tp_hash
// ---------------------------------------------------------------------------

#[pyclass]
struct FormatAndHash;

#[pymethods]
impl FormatAndHash {
    #[new]
    fn new() -> Self {
        FormatAndHash
    }
    fn __repr__(&self) -> &'static str {
        "FormatAndHash()"
    }
    fn __str__(&self) -> &'static str {
        "FormatAndHash"
    }
    fn __hash__(&self) -> isize {
        42
    }
}

#[test]
fn unary_format_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<FormatAndHash>();
        py_expect_exception!(py, cls, "cls.__repr__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__str__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__hash__(object())", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// tp_richcompare (via per-comparison SlotFragmentDef entries)
// ---------------------------------------------------------------------------

#[pyclass]
struct RichCmp;

#[pymethods]
impl RichCmp {
    #[new]
    fn new() -> Self {
        RichCmp
    }
    fn __lt__(&self, _other: i32) -> bool {
        false
    }
    fn __le__(&self, _other: i32) -> bool {
        false
    }
    fn __eq__(&self, _other: i32) -> bool {
        false
    }
    fn __ne__(&self, _other: i32) -> bool {
        true
    }
    fn __gt__(&self, _other: i32) -> bool {
        false
    }
    fn __ge__(&self, _other: i32) -> bool {
        false
    }
}

#[test]
fn richcmp_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<RichCmp>();
        py_expect_exception!(py, cls, "cls.__lt__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__le__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__eq__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__ne__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__gt__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__ge__(object(), 0)", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// tp_iter / tp_iternext
// ---------------------------------------------------------------------------

#[pyclass]
struct IterSlots;

#[pymethods]
impl IterSlots {
    #[new]
    fn new() -> Self {
        IterSlots
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(&mut self) -> Option<i32> {
        None
    }
}

#[test]
fn iter_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<IterSlots>();
        py_expect_exception!(py, cls, "cls.__iter__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__next__(object())", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// mp_length / mp_subscript / mp_ass_subscript / sq_contains
// ---------------------------------------------------------------------------

#[pyclass]
struct ContainerSlots;

#[pymethods]
impl ContainerSlots {
    #[new]
    fn new() -> Self {
        ContainerSlots
    }
    fn __len__(&self) -> usize {
        0
    }
    fn __getitem__(&self, _key: i32) -> i32 {
        0
    }
    fn __setitem__(&mut self, _key: i32, _val: i32) {}
    fn __delitem__(&mut self, _key: i32) {}
    fn __contains__(&self, _item: i32) -> bool {
        false
    }
}

#[test]
fn container_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<ContainerSlots>();
        py_expect_exception!(py, cls, "cls.__len__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__getitem__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__setitem__(object(), 0, 1)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__delitem__(object(), 0)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__contains__(object(), 0)", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// tp_setattro (via __setattr__ / __delattr__ SlotFragmentDef entries)
// ---------------------------------------------------------------------------

#[pyclass]
struct AttrSlots {
    _value: i32,
}

#[pymethods]
impl AttrSlots {
    #[new]
    fn new() -> Self {
        AttrSlots { _value: 0 }
    }
    fn __setattr__(&mut self, _attr: &str, _val: i32) {}
    fn __delattr__(&mut self, _attr: &str) {}
}

#[test]
fn setattr_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<AttrSlots>();
        py_expect_exception!(py, cls, "cls.__setattr__(object(), 'x', 1)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__delattr__(object(), 'x')", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// tp_call
// ---------------------------------------------------------------------------

#[pyclass]
struct CallSlot;

#[pymethods]
impl CallSlot {
    #[new]
    fn new() -> Self {
        CallSlot
    }
    fn __call__(&self) -> i32 {
        0
    }
}

#[test]
fn call_slot_rejects_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<CallSlot>();
        py_expect_exception!(py, cls, "cls.__call__(object())", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// nb_positive / nb_negative / nb_invert / nb_int / nb_float / nb_index / nb_bool
// ---------------------------------------------------------------------------

#[pyclass]
struct NumericUnary;

#[pymethods]
impl NumericUnary {
    #[new]
    fn new() -> Self {
        NumericUnary
    }
    fn __pos__(&self) -> i32 {
        0
    }
    fn __neg__(&self) -> i32 {
        0
    }
    fn __invert__(&self) -> i32 {
        0
    }
    fn __int__(&self) -> i32 {
        0
    }
    fn __float__(&self) -> f64 {
        0.0
    }
    fn __index__(&self) -> i32 {
        0
    }
    fn __bool__(&self) -> bool {
        false
    }
}

#[test]
fn numeric_unary_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<NumericUnary>();
        py_expect_exception!(py, cls, "cls.__pos__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__neg__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__invert__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__int__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__float__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__index__(object())", PyTypeError);
        py_expect_exception!(py, cls, "cls.__bool__(object())", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// nb_inplace_add / nb_inplace_subtract / nb_inplace_multiply
// (SlotDef::binary_inplace_operator — Trusted self, return_self)
// ---------------------------------------------------------------------------

#[pyclass]
struct InplaceOps;

#[pymethods]
impl InplaceOps {
    #[new]
    fn new() -> Self {
        InplaceOps
    }
    fn __iadd__(&mut self, _other: i32) {}
    fn __isub__(&mut self, _other: i32) {}
    fn __imul__(&mut self, _other: i32) {}
}

#[test]
fn inplace_operator_slots_reject_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<InplaceOps>();
        py_expect_exception!(py, cls, "cls.__iadd__(object(), 1)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__isub__(object(), 1)", PyTypeError);
        py_expect_exception!(py, cls, "cls.__imul__(object(), 1)", PyTypeError);
    });
}

// ---------------------------------------------------------------------------
// tp_getset (getter / setter installed via PyGetSetDef)
// ---------------------------------------------------------------------------

#[pyclass]
struct GetSetSlots {
    value: i32,
}

#[pymethods]
impl GetSetSlots {
    #[new]
    fn new() -> Self {
        GetSetSlots { value: 0 }
    }
    // pyo3 strips the `get_`/`set_` prefix to derive the Python attribute name `prop`.
    #[getter]
    fn get_prop(&self) -> i32 {
        self.value
    }
    #[setter]
    fn set_prop(&mut self, v: i32) {
        self.value = v;
    }
}

#[test]
fn getset_descriptor_rejects_wrong_receiver() {
    Python::attach(|py| {
        let cls = py.get_type::<GetSetSlots>();
        // `cls.__dict__['prop']` gives the raw getset_descriptor without going
        // through the descriptor protocol.  Calling __get__/__set__ on it with a
        // wrong-type instance goes through CPython's `descr_check`, which raises
        // TypeError before our Rust getter/setter code is ever reached.
        py_expect_exception!(
            py,
            cls,
            "cls.__dict__['prop'].__get__(object(), type(object()))",
            PyTypeError
        );
        py_expect_exception!(
            py,
            cls,
            "cls.__dict__['prop'].__set__(object(), 1)",
            PyTypeError
        );
    });
}
