// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol

use ffi;
use err::PyResult;
use callback::PyObjectCallbackConverter;
use typeob::PyTypeInfo;
use class::methods::PyMethodDef;
use class::basic::PyObjectProtocolImpl;
use {IntoPyObject, FromPyObject};

/// Number interface
#[allow(unused_variables)]
pub trait PyNumberProtocol<'p>: PyTypeInfo {

    fn __add__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberAddProtocol<'p> { unimplemented!() }
    fn __sub__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberSubProtocol<'p> { unimplemented!() }
    fn __mul__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberMulProtocol<'p> { unimplemented!() }
    fn __matmul__(lhs: Self::Left, rhs: Self::Right)
                  -> Self::Result where Self: PyNumberMatmulProtocol<'p> { unimplemented!() }
    fn __truediv__(lhs: Self::Left, rhs: Self::Right)
                   -> Self::Result where Self: PyNumberTruedivProtocol<'p> { unimplemented!() }
    fn __floordiv__(lhs: Self::Left, rhs: Self::Right)
                    -> Self::Result where Self: PyNumberFloordivProtocol<'p> { unimplemented!() }
    fn __mod__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberModProtocol<'p> { unimplemented!() }
    fn __divmod__(lhs: Self::Left, rhs: Self::Right)
                  -> Self::Result where Self: PyNumberDivmodProtocol<'p> { unimplemented!() }
    fn __pow__(lhs: Self::Left, rhs: Self::Right, modulo: Self::Modulo)
               -> Self::Result where Self: PyNumberPowProtocol<'p> { unimplemented!() }
    fn __lshift__(lhs: Self::Left, rhs: Self::Right)
                  -> Self::Result where Self: PyNumberLShiftProtocol<'p> { unimplemented!() }
    fn __rshift__(lhs: Self::Left, rhs: Self::Right)
                  -> Self::Result where Self: PyNumberRShiftProtocol<'p> { unimplemented!() }
    fn __and__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberAndProtocol<'p> { unimplemented!() }
    fn __xor__(lhs: Self::Left, rhs: Self::Right)
               -> Self::Result where Self: PyNumberXorProtocol<'p> { unimplemented!() }
    fn __or__(lhs: Self::Left, rhs: Self::Right)
              -> Self::Result where Self: PyNumberOrProtocol<'p> { unimplemented!() }

    fn __radd__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRAddProtocol<'p> { unimplemented!() }
    fn __rsub__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRSubProtocol<'p> { unimplemented!() }
    fn __rmul__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRMulProtocol<'p> { unimplemented!() }
    fn __rmatmul__(&'p self, other: Self::Other)
                   -> Self::Result where Self: PyNumberRMatmulProtocol<'p> { unimplemented!() }
    fn __rtruediv__(&'p self, other: Self::Other)
                    -> Self::Result where Self: PyNumberRTruedivProtocol<'p> { unimplemented!() }
    fn __rfloordiv__(&'p self, other: Self::Other)
                     -> Self::Result where Self: PyNumberRFloordivProtocol<'p> { unimplemented!() }
    fn __rmod__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRModProtocol<'p> { unimplemented!() }
    fn __rdivmod__(&'p self, other: Self::Other)
                   -> Self::Result where Self: PyNumberRDivmodProtocol<'p> { unimplemented!() }
    fn __rpow__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRPowProtocol<'p> { unimplemented!() }
    fn __rlshift__(&'p self, other: Self::Other)
                   -> Self::Result where Self: PyNumberRLShiftProtocol<'p> { unimplemented!() }
    fn __rrshift__(&'p self, other: Self::Other)
                   -> Self::Result where Self: PyNumberRRShiftProtocol<'p> { unimplemented!() }
    fn __rand__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRAndProtocol<'p> { unimplemented!() }
    fn __rxor__(&'p self, other: Self::Other)
                -> Self::Result where Self: PyNumberRXorProtocol<'p> { unimplemented!() }
    fn __ror__(&'p self, other: Self::Other)
               -> Self::Result where Self: PyNumberROrProtocol<'p> { unimplemented!() }

    fn __iadd__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberIAddProtocol<'p> { unimplemented!() }
    fn __isub__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberISubProtocol<'p> { unimplemented!() }
    fn __imul__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberIMulProtocol<'p> { unimplemented!() }
    fn __imatmul__(&'p mut self, other: Self::Other)
                   -> Self::Result where Self: PyNumberIMatmulProtocol<'p> { unimplemented!() }
    fn __itruediv__(&'p mut self, other: Self::Other)
                    -> Self::Result where Self: PyNumberITruedivProtocol<'p> {unimplemented!()}
    fn __ifloordiv__(&'p mut self, other: Self::Other)
                     -> Self::Result where Self: PyNumberIFloordivProtocol<'p> {unimplemented!() }
    fn __imod__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberIModProtocol<'p> { unimplemented!() }
    fn __ipow__(&'p mut self, other: Self::Other, modulo: Self::Modulo)
                -> Self::Result where Self: PyNumberIPowProtocol<'p> { unimplemented!() }
    fn __ilshift__(&'p mut self, other: Self::Other)
                   -> Self::Result where Self: PyNumberILShiftProtocol<'p> { unimplemented!() }
    fn __irshift__(&'p mut self, other: Self::Other)
                   -> Self::Result where Self: PyNumberIRShiftProtocol<'p> { unimplemented!() }
    fn __iand__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberIAndProtocol<'p> { unimplemented!() }
    fn __ixor__(&'p mut self, other: Self::Other)
                -> Self::Result where Self: PyNumberIXorProtocol<'p> { unimplemented!() }
    fn __ior__(&'p mut self, other: Self::Other)
               -> Self::Result where Self: PyNumberIOrProtocol<'p> { unimplemented!() }

    // Unary arithmetic
    fn __neg__(&'p self)
               -> Self::Result where Self: PyNumberNegProtocol<'p> { unimplemented!() }
    fn __pos__(&'p self)
               -> Self::Result where Self: PyNumberPosProtocol<'p> { unimplemented!() }
    fn __abs__(&'p self)
               -> Self::Result where Self: PyNumberAbsProtocol<'p> { unimplemented!() }
    fn __invert__(&'p self)
                  -> Self::Result where Self: PyNumberInvertProtocol<'p> { unimplemented!() }
    fn __complex__(&'p self)
                   -> Self::Result where Self: PyNumberComplexProtocol<'p> { unimplemented!() }
    fn __int__(&'p self)
               -> Self::Result where Self: PyNumberIntProtocol<'p> { unimplemented!() }
    fn __float__(&'p self)
                 -> Self::Result where Self: PyNumberFloatProtocol<'p> { unimplemented!() }
    fn __round__(&'p self)
                 -> Self::Result where Self: PyNumberRoundProtocol<'p> { unimplemented!() }
    fn __index__(&'p self)
                 -> Self::Result where Self: PyNumberIndexProtocol<'p> { unimplemented!() }
}


pub trait PyNumberAddProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberSubProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberMulProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberTruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberModProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberPowProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberLShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberAndProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberXorProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberOrProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}


pub trait PyNumberRAddProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRSubProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRMulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRTruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRModProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRPowProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRLShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRAndProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRXorProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberROrProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyNumberIAddProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberISubProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIMulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberITruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIModProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIPowProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberILShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIAndProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIXorProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}
pub trait PyNumberIOrProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PyNumberNegProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberPosProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberAbsProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberInvertProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberComplexProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIntProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberFloatProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRoundProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIndexProtocol<'p>: PyNumberProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}



#[doc(hidden)]
pub trait PyNumberProtocolImpl {
    fn methods() -> Vec<PyMethodDef>;
    fn tp_as_number() -> Option<ffi::PyNumberMethods>;
}

impl<'p, T> PyNumberProtocolImpl for T {
    #[cfg(Py_3)]
    default fn tp_as_number() -> Option<ffi::PyNumberMethods> {
        if let Some(nb_bool) = <Self as PyObjectProtocolImpl>::nb_bool_fn() {
            let meth = ffi::PyNumberMethods {
                nb_bool: Some(nb_bool),
                ..
                ffi::PyNumberMethods_INIT
            };
            Some(meth)
        } else {
            None
        }
    }
    #[cfg(not(Py_3))]
    default fn tp_as_number() -> Option<ffi::PyNumberMethods> {
        if let Some(nb_bool) = <Self as PyObjectProtocolImpl>::nb_bool_fn() {
            let meth = ffi::PyNumberMethods {
                nb_nonzero: Some(nb_bool),
                ..
                ffi::PyNumberMethods_INIT
            };
            Some(meth)
        } else {
            None
        }
    }
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<'p, T> PyNumberProtocolImpl for T where T: PyNumberProtocol<'p> {
    #[cfg(Py_3)]
    fn tp_as_number() -> Option<ffi::PyNumberMethods> {
        Some(ffi::PyNumberMethods {
            nb_add: Self::nb_add(),
            nb_subtract: Self::nb_subtract(),
            nb_multiply: Self::nb_multiply(),
            nb_remainder: Self::nb_remainder(),
            nb_divmod: Self::nb_divmod(),
            nb_power: Self::nb_power(),
            nb_negative: Self::nb_negative(),
            nb_positive: Self::nb_positive(),
            nb_absolute: Self::nb_absolute(),
            nb_bool: <Self as PyObjectProtocolImpl>::nb_bool_fn(),
            nb_invert: Self::nb_invert(),
            nb_lshift: Self::nb_lshift(),
            nb_rshift: Self::nb_rshift(),
            nb_and: Self::nb_and(),
            nb_xor: Self::nb_xor(),
            nb_or: Self::nb_or(),
            nb_int: Self::nb_int(),
            nb_reserved: ::std::ptr::null_mut(),
            nb_float: Self::nb_float(),
            nb_inplace_add: Self::nb_inplace_add(),
            nb_inplace_subtract: Self::nb_inplace_subtract(),
            nb_inplace_multiply: Self::nb_inplace_multiply(),
            nb_inplace_remainder: Self::nb_inplace_remainder(),
            nb_inplace_power: Self::nb_inplace_power(),
            nb_inplace_lshift: Self::nb_inplace_lshift(),
            nb_inplace_rshift: Self::nb_inplace_rshift(),
            nb_inplace_and: Self::nb_inplace_and(),
            nb_inplace_xor: Self::nb_inplace_xor(),
            nb_inplace_or: Self::nb_inplace_or(),
            nb_floor_divide: Self::nb_floor_divide(),
            nb_true_divide: Self::nb_true_divide(),
            nb_inplace_floor_divide: Self::nb_inplace_floor_divide(),
            nb_inplace_true_divide: Self::nb_inplace_true_divide(),
            nb_index: Self::nb_index(),
            nb_matrix_multiply: Self::nb_matrix_multiply(),
            nb_inplace_matrix_multiply: Self::nb_inplace_matrix_multiply(),
        })
    }
    #[cfg(not(Py_3))]
    fn tp_as_number() -> Option<ffi::PyNumberMethods> {
        Some(ffi::PyNumberMethods {
            nb_add: Self::nb_add(),
            nb_subtract: Self::nb_subtract(),
            nb_multiply: Self::nb_multiply(),
            nb_remainder: Self::nb_remainder(),
            nb_divmod: Self::nb_divmod(),
            nb_power: Self::nb_power(),
            nb_negative: Self::nb_negative(),
            nb_positive: Self::nb_positive(),
            nb_absolute: Self::nb_absolute(),
            nb_nonzero: <Self as PyObjectProtocolImpl>::nb_bool_fn(),
            nb_invert: Self::nb_invert(),
            nb_lshift: Self::nb_lshift(),
            nb_rshift: Self::nb_rshift(),
            nb_and: Self::nb_and(),
            nb_xor: Self::nb_xor(),
            nb_or: Self::nb_or(),
            nb_c_int: Self::nb_int(),
            nb_float: Self::nb_float(),
            nb_inplace_add: Self::nb_inplace_add(),
            nb_inplace_subtract: Self::nb_inplace_subtract(),
            nb_inplace_multiply: Self::nb_inplace_multiply(),
            nb_inplace_remainder: Self::nb_inplace_remainder(),
            nb_inplace_power: Self::nb_inplace_power(),
            nb_inplace_lshift: Self::nb_inplace_lshift(),
            nb_inplace_rshift: Self::nb_inplace_rshift(),
            nb_inplace_and: Self::nb_inplace_and(),
            nb_inplace_xor: Self::nb_inplace_xor(),
            nb_inplace_or: Self::nb_inplace_or(),
            nb_floor_divide: Self::nb_floor_divide(),
            nb_true_divide: Self::nb_true_divide(),
            nb_inplace_floor_divide: Self::nb_inplace_floor_divide(),
            nb_inplace_true_divide: Self::nb_inplace_true_divide(),
            nb_index: Self::nb_index(),
            nb_coerce: None,
            nb_divide: None,
            nb_hex: None,
            nb_inplace_divide: None,
            nb_long: None,
            nb_oct: None,
        })
    }

    #[inline]
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as PyNumberRAddProtocolImpl>::__radd__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRSubProtocolImpl>::__rsub__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRMulProtocolImpl>::__rmul__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRMatmulProtocolImpl>::__rmatmul__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRTruedivProtocolImpl>::__rtruediv__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRFloordivProtocolImpl>::__rfloordiv__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRModProtocolImpl>::__rmod__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRDivmodProtocolImpl>::__rdivmod__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRPowProtocolImpl>::__rpow__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRLShiftProtocolImpl>::__rlshift__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRRShiftProtocolImpl>::__rrshift__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRAndProtocolImpl>::__rand__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRXorProtocolImpl>::__rxor__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberROrProtocolImpl>::__ror__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberComplexProtocolImpl>::__complex__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyNumberRoundProtocolImpl>::__round__() {
            methods.push(def)
        }

        methods
    }
}

trait PyNumberAddProtocolImpl {
    fn nb_add() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberAddProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_add() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberAddProtocolImpl for T where T: for<'p> PyNumberAddProtocol<'p> {
    fn nb_add() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberAddProtocol, T::__add__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberSubProtocolImpl {
    fn nb_subtract() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberSubProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_subtract() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberSubProtocolImpl for T where T: for<'p> PyNumberSubProtocol<'p> {
    fn nb_subtract() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberSubProtocol, T::__sub__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberMulProtocolImpl {
    fn nb_multiply() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberMulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberMulProtocolImpl for T where T: for<'p> PyNumberMulProtocol<'p> {
    fn nb_multiply() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberMulProtocol, T::__mul__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberMatmulProtocolImpl {
    fn nb_matrix_multiply() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberMatmulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_matrix_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberMatmulProtocolImpl for T where T: for<'p> PyNumberMatmulProtocol<'p> {
    fn nb_matrix_multiply() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberMatmulProtocol, T::__matmul__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberTruedivProtocolImpl {
    fn nb_true_divide() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberTruedivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_true_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberTruedivProtocolImpl for T
    where T: for<'p> PyNumberTruedivProtocol<'p>
{
    fn nb_true_divide() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberTruedivProtocol, T::__truediv__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberFloordivProtocolImpl {
    fn nb_floor_divide() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberFloordivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_floor_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberFloordivProtocolImpl for T
    where T: for<'p> PyNumberFloordivProtocol<'p>
{
    fn nb_floor_divide() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberFloordivProtocol, T::__floordiv__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberModProtocolImpl {
    fn nb_remainder() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberModProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_remainder() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberModProtocolImpl for T where T: for<'p> PyNumberModProtocol<'p> {
    fn nb_remainder() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberModProtocol, T::__mod__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberDivmodProtocolImpl {
    fn nb_divmod() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberDivmodProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_divmod() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberDivmodProtocolImpl for T where T: for<'p> PyNumberDivmodProtocol<'p> {
    fn nb_divmod() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberDivmodProtocol, T::__divmod__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberPowProtocolImpl {
    fn nb_power() -> Option<ffi::ternaryfunc>;
}
impl<'p, T> PyNumberPowProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_power() -> Option<ffi::ternaryfunc> {None}
}
impl<T> PyNumberPowProtocolImpl for T where T: for<'p> PyNumberPowProtocol<'p> {
    fn nb_power() -> Option<ffi::ternaryfunc> {
        py_ternary_num_func!(
            PyNumberPowProtocol,
            T::__pow__, <T as PyNumberPowProtocol>::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberLShiftProtocolImpl {
    fn nb_lshift() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberLShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_lshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberLShiftProtocolImpl for T where T: for<'p> PyNumberLShiftProtocol<'p> {
    fn nb_lshift() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberLShiftProtocol, T::__lshift__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberRShiftProtocolImpl {
    fn nb_rshift() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberRShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_rshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberRShiftProtocolImpl for T where T: for<'p> PyNumberRShiftProtocol<'p> {
    fn nb_rshift() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberRShiftProtocol, T::__rshift__, T::Success, PyObjectCallbackConverter)
    }
}


trait PyNumberAndProtocolImpl {
    fn nb_and() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberAndProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_and() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberAndProtocolImpl for T where T: for<'p> PyNumberAndProtocol<'p> {
    fn nb_and() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberAndProtocol, T::__and__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberXorProtocolImpl {
    fn nb_xor() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberXorProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_xor() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberXorProtocolImpl for T where T: for<'p> PyNumberXorProtocol<'p> {
    fn nb_xor() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberXorProtocol, T::__xor__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberOrProtocolImpl {
    fn nb_or() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberOrProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_or() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberOrProtocolImpl for T where T: for<'p> PyNumberOrProtocol<'p> {
    fn nb_or() -> Option<ffi::binaryfunc> {
        py_binary_num_func!(
            PyNumberOrProtocol, T::__or__, T::Success, PyObjectCallbackConverter)
    }
}


trait PyNumberIAddProtocolImpl {
    fn nb_inplace_add() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIAddProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_add() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIAddProtocolImpl for T where T: for<'p> PyNumberIAddProtocol<'p> {
    fn nb_inplace_add() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIAddProtocol, T::__iadd__)
    }
}

trait PyNumberISubProtocolImpl {
    fn nb_inplace_subtract() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberISubProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_subtract() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberISubProtocolImpl for T where T: for<'p> PyNumberISubProtocol<'p> {
    fn nb_inplace_subtract() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberISubProtocol, T::__isub__)
    }
}

trait PyNumberIMulProtocolImpl {
    fn nb_inplace_multiply() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIMulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIMulProtocolImpl for T where T: for<'p> PyNumberIMulProtocol<'p> {
    fn nb_inplace_multiply() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIMulProtocol, T::__imul__)
    }
}

trait PyNumberIMatmulProtocolImpl {
    fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIMatmulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIMatmulProtocolImpl for T
    where T: for<'p> PyNumberIMatmulProtocol<'p>
{
    fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIMatmulProtocol, T::__imatmul__)
    }
}

trait PyNumberITruedivProtocolImpl {
    fn nb_inplace_true_divide() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberITruedivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_true_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberITruedivProtocolImpl for T
    where T: for<'p> PyNumberITruedivProtocol<'p>
{
    fn nb_inplace_true_divide() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberITruedivProtocol, T::__itruediv__)
    }
}

trait PyNumberIFloordivProtocolImpl {
    fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIFloordivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIFloordivProtocolImpl for T
    where T: for<'p> PyNumberIFloordivProtocol<'p>
{
    fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIFloordivProtocol, T::__ifloordiv__)
    }
}

trait PyNumberIModProtocolImpl {
    fn nb_inplace_remainder() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIModProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_remainder() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIModProtocolImpl for T where T: for<'p> PyNumberIModProtocol<'p> {
    fn nb_inplace_remainder() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIModProtocol, T::__imod__)
    }
}

trait PyNumberIPowProtocolImpl {
    fn nb_inplace_power() -> Option<ffi::ternaryfunc>;
}
impl<'p, T> PyNumberIPowProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_power() -> Option<ffi::ternaryfunc> {None}
}
impl<T> PyNumberIPowProtocolImpl for T where T: for<'p> PyNumberIPowProtocol<'p> {
    fn nb_inplace_power() -> Option<ffi::ternaryfunc> {
        py_ternary_self_func!(PyNumberIPowProtocol, T::__ipow__)
    }
}

trait PyNumberILShiftProtocolImpl {
    fn nb_inplace_lshift() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberILShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_lshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberILShiftProtocolImpl for T
    where T: for<'p> PyNumberILShiftProtocol<'p>
{
    fn nb_inplace_lshift() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberILShiftProtocol, T::__ilshift__)
    }
}

trait PyNumberIRShiftProtocolImpl {
    fn nb_inplace_rshift() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIRShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_rshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIRShiftProtocolImpl for T
    where T: for<'p> PyNumberIRShiftProtocol<'p>
{
    fn nb_inplace_rshift() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIRShiftProtocol, T::__irshift__)
    }
}


trait PyNumberIAndProtocolImpl {
    fn nb_inplace_and() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIAndProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_and() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIAndProtocolImpl for T where T: for<'p> PyNumberIAndProtocol<'p> {
    fn nb_inplace_and() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIAndProtocol, T::__iand__)
    }
}

trait PyNumberIXorProtocolImpl {
    fn nb_inplace_xor() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIXorProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_xor() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIXorProtocolImpl for T where T: for<'p> PyNumberIXorProtocol<'p> {
    fn nb_inplace_xor() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIXorProtocol, T::__ixor__)
    }
}

trait PyNumberIOrProtocolImpl {
    fn nb_inplace_or() -> Option<ffi::binaryfunc>;
}
impl<'p, T> PyNumberIOrProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_inplace_or() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIOrProtocolImpl for T where T: for<'p> PyNumberIOrProtocol<'p> {
    fn nb_inplace_or() -> Option<ffi::binaryfunc> {
        py_binary_self_func!(PyNumberIOrProtocol, T::__ior__)
    }
}


trait PyNumberRAddProtocolImpl {
    fn __radd__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRAddProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __radd__() -> Option<PyMethodDef> {None}
}

trait PyNumberRSubProtocolImpl {
    fn __rsub__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRSubProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rsub__() -> Option<PyMethodDef> {None}
}

trait PyNumberRMulProtocolImpl {
    fn __rmul__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRMulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rmul__() -> Option<PyMethodDef> {None}
}

trait PyNumberRMatmulProtocolImpl {
    fn __rmatmul__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRMatmulProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rmatmul__() -> Option<PyMethodDef> {None}
}

trait PyNumberRTruedivProtocolImpl {
    fn __rtruediv__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRTruedivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rtruediv__() -> Option<PyMethodDef> {None}
}

trait PyNumberRFloordivProtocolImpl {
    fn __rfloordiv__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRFloordivProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rfloordiv__() -> Option<PyMethodDef> {None}
}

trait PyNumberRModProtocolImpl {
    fn __rmod__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRModProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rmod__() -> Option<PyMethodDef> {None}
}

trait PyNumberRDivmodProtocolImpl {
    fn __rdivmod__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRDivmodProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rdivmod__() -> Option<PyMethodDef> {None}
}

trait PyNumberRPowProtocolImpl {
    fn __rpow__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRPowProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rpow__() -> Option<PyMethodDef> {None}
}

trait PyNumberRLShiftProtocolImpl {
    fn __rlshift__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRLShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rlshift__() -> Option<PyMethodDef> {None}
}

trait PyNumberRRShiftProtocolImpl {
    fn __rrshift__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRRShiftProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rrshift__() -> Option<PyMethodDef> {None}
}


trait PyNumberRAndProtocolImpl {
    fn __rand__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRAndProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rand__() -> Option<PyMethodDef> {None}
}

trait PyNumberRXorProtocolImpl {
    fn __rxor__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRXorProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __rxor__() -> Option<PyMethodDef> {None}
}

trait PyNumberROrProtocolImpl {
    fn __ror__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberROrProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __ror__() -> Option<PyMethodDef> {None}
}

trait PyNumberNegProtocolImpl {
    fn nb_negative() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberNegProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_negative() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberNegProtocolImpl for T where T: for<'p> PyNumberNegProtocol<'p>
{
    #[inline]
    fn nb_negative() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberNegProtocol, T::__neg__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberPosProtocolImpl {
    fn nb_positive() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberPosProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_positive() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberPosProtocolImpl for T where T: for<'p> PyNumberPosProtocol<'p>
{
    fn nb_positive() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberPosProtocol, T::__pos__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberAbsProtocolImpl {
    fn nb_absolute() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberAbsProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_absolute() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberAbsProtocolImpl for T where T: for<'p> PyNumberAbsProtocol<'p>
{
    fn nb_absolute() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberAbsProtocol, T::__abs__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberInvertProtocolImpl {
    fn nb_invert() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberInvertProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_invert() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberInvertProtocolImpl for T where T: for<'p> PyNumberInvertProtocol<'p>
{
    fn nb_invert() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberInvertProtocol, T::__invert__,
                       T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberIntProtocolImpl {
    fn nb_int() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberIntProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_int() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberIntProtocolImpl for T where T: for<'p> PyNumberIntProtocol<'p>
{
    fn nb_int() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberIntProtocol, T::__int__,
                       T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberFloatProtocolImpl {
    fn nb_float() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberFloatProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_float() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberFloatProtocolImpl for T where T: for<'p> PyNumberFloatProtocol<'p>
{
    fn nb_float() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberFloatProtocol, T::__float__,
                       T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberIndexProtocolImpl {
    fn nb_index() -> Option<ffi::unaryfunc>;
}
impl<'p, T> PyNumberIndexProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn nb_index() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberIndexProtocolImpl for T
    where T: for<'p> PyNumberIndexProtocol<'p>
{
    fn nb_index() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyNumberIndexProtocol,
                       T::__index__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyNumberComplexProtocolImpl {
    fn __complex__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberComplexProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __complex__() -> Option<PyMethodDef> {None}
}

trait PyNumberRoundProtocolImpl {
    fn __round__() -> Option<PyMethodDef>;
}
impl<'p, T> PyNumberRoundProtocolImpl for T where T: PyNumberProtocol<'p> {
    default fn __round__() -> Option<PyMethodDef> {None}
}
