// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use callback::{PyObjectCallbackConverter};
// use class::basic::{PyObjectProtocol, PyObjectProtocolImpl};
use class::methods::PyMethodDef;
use class::basic::PyObjectProtocolImpl;
use ::{c_void, ToPyObject, FromPyObject};

/// Number interface
#[allow(unused_variables)]
pub trait PyNumberProtocol: PythonObject {

    fn __add__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberAddProtocol { unimplemented!() }
    fn __sub__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberSubProtocol { unimplemented!() }
    fn __mul__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberMulProtocol { unimplemented!() }
    fn __matmul__(&self, py: Python, other: Self::Other)
                  -> Self::Result where Self: PyNumberMatmulProtocol { unimplemented!() }
    fn __truediv__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberTruedivProtocol { unimplemented!() }
    fn __floordiv__(&self, py: Python, other: Self::Other)
                    -> Self::Result where Self: PyNumberFloordivProtocol { unimplemented!() }
    fn __mod__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberModProtocol { unimplemented!() }
    fn __divmod__(&self, py: Python, other: Self::Other)
                  -> Self::Result where Self: PyNumberDivmodProtocol { unimplemented!() }
    fn __pow__(&self, py: Python, other: Self::Other, modulo: Self::Modulo)
               -> Self::Result where Self: PyNumberPowProtocol { unimplemented!() }
    fn __lshift__(&self, py: Python, other: Self::Other)
                  -> Self::Result where Self: PyNumberLShiftProtocol { unimplemented!() }
    fn __rshift__(&self, py: Python, other: Self::Other)
                  -> Self::Result where Self: PyNumberRShiftProtocol { unimplemented!() }
    fn __and__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberAndProtocol { unimplemented!() }
    fn __xor__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberXorProtocol { unimplemented!() }
    fn __or__(&self, py: Python, other: Self::Other)
              -> Self::Result where Self: PyNumberOrProtocol { unimplemented!() }

    fn __radd__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRAddProtocol { unimplemented!() }
    fn __rsub__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRSubProtocol { unimplemented!() }
    fn __rmul__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRMulProtocol { unimplemented!() }
    fn __rmatmul__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberRMatmulProtocol { unimplemented!() }
    fn __rtruediv__(&self, py: Python, other: Self::Other)
                    -> Self::Result where Self: PyNumberRTruedivProtocol { unimplemented!() }
    fn __rfloordiv__(&self, py: Python, other: Self::Other)
                     -> Self::Result where Self: PyNumberRFloordivProtocol { unimplemented!() }
    fn __rmod__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRModProtocol { unimplemented!() }
    fn __rdivmod__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberRDivmodProtocol { unimplemented!() }
    fn __rpow__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRPowProtocol { unimplemented!() }
    fn __rlshift__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberRLShiftProtocol { unimplemented!() }
    fn __rrshift__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberRRShiftProtocol { unimplemented!() }
    fn __rand__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRAndProtocol { unimplemented!() }
    fn __rxor__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberRXorProtocol { unimplemented!() }
    fn __ror__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberROrProtocol { unimplemented!() }

    fn __iadd__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberIAddProtocol { unimplemented!() }
    fn __isub__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberISubProtocol { unimplemented!() }
    fn __imul__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberIMulProtocol { unimplemented!() }
    fn __imatmul__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberIMatmulProtocol { unimplemented!() }
    fn __itruediv__(&self, py: Python, other: Self::Other)
                    -> Self::Result where Self: PyNumberITruedivProtocol { unimplemented!() }
    fn __ifloordiv__(&self, py: Python, other: Self::Other)
                     -> Self::Result where Self: PyNumberIFloordivProtocol { unimplemented!() }
    fn __imod__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberIModProtocol { unimplemented!() }
    fn __ipow__(&self, py: Python, other: Self::Other, modulo: Self::Modulo)
                -> Self::Result where Self: PyNumberIPowProtocol { unimplemented!() }
    fn __ilshift__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberILShiftProtocol { unimplemented!() }
    fn __irshift__(&self, py: Python, other: Self::Other)
                   -> Self::Result where Self: PyNumberIRShiftProtocol { unimplemented!() }
    fn __iand__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberIAndProtocol { unimplemented!() }
    fn __ixor__(&self, py: Python, other: Self::Other)
                -> Self::Result where Self: PyNumberIXorProtocol { unimplemented!() }
    fn __ior__(&self, py: Python, other: Self::Other)
               -> Self::Result where Self: PyNumberIOrProtocol { unimplemented!() }

    // Unary arithmetic
    fn __neg__(&self, py: Python)
               -> Self::Result where Self: PyNumberNegProtocol { unimplemented!() }
    fn __pos__(&self, py: Python)
               -> Self::Result where Self: PyNumberPosProtocol { unimplemented!() }
    fn __abs__(&self, py: Python)
               -> Self::Result where Self: PyNumberAbsProtocol { unimplemented!() }
    fn __invert__(&self, py: Python)
                  -> Self::Result where Self: PyNumberInvertProtocol { unimplemented!() }
    fn __complex__(&self, py: Python)
                   -> Self::Result where Self: PyNumberComplexProtocol { unimplemented!() }
    fn __int__(&self, py: Python)
               -> Self::Result where Self: PyNumberIntProtocol { unimplemented!() }
    fn __float__(&self, py: Python)
                 -> Self::Result where Self: PyNumberFloatProtocol { unimplemented!() }
    fn __round__(&self, py: Python)
                 -> Self::Result where Self: PyNumberRoundProtocol { unimplemented!() }
    fn __index__(&self, py: Python)
                 -> Self::Result where Self: PyNumberIndexProtocol { unimplemented!() }
}


pub trait PyNumberAddProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberSubProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberMulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberMatmulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberTruedivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberFloordivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberModProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberDivmodProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberPowProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Modulo: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberLShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberAndProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberXorProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberOrProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}


pub trait PyNumberRAddProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRSubProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRMulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRMatmulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRTruedivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRFloordivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRModProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRDivmodProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRPowProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Modulo: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRLShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRRShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRAndProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRXorProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberROrProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyNumberIAddProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberISubProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIMulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIMatmulProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberITruedivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIFloordivProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIModProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIDivmodProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIPowProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Modulo: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberILShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIRShiftProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIAndProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIXorProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIOrProtocol: PyNumberProtocol {
    type Other: for<'a> FromPyObject<'a>;
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyNumberNegProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberPosProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberAbsProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberInvertProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberComplexProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIntProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberFloatProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberRoundProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}
pub trait PyNumberIndexProtocol: PyNumberProtocol {
    type Success: ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}



#[doc(hidden)]
pub trait PyNumberProtocolImpl {
    fn methods() -> Vec<PyMethodDef>;
    fn tp_as_number() -> Option<ffi::PyNumberMethods>;
}

impl<T> PyNumberProtocolImpl for T {
    default fn tp_as_number() -> Option<ffi::PyNumberMethods> {
        if let Some(nb_bool) = <Self as PyObjectProtocolImpl>::nb_bool_fn() {
            let mut meth = ffi::PyNumberMethods_INIT;
            meth.nb_bool = Some(nb_bool);
            Some(meth)
        } else {
            None
        }
    }
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<T> PyNumberProtocolImpl for T where T: PyNumberProtocol {
    #[inline]
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
            nb_reserved: 0 as *mut c_void,
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
impl<T> PyNumberAddProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_add() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberAddProtocolImpl for T where T: PyNumberAddProtocol {
    fn nb_add() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberAddProtocol, T::__add__, PyObjectCallbackConverter)
    }
}

trait PyNumberSubProtocolImpl {
    fn nb_subtract() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberSubProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_subtract() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberSubProtocolImpl for T where T: PyNumberSubProtocol {
    fn nb_subtract() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberSubProtocol, T::__sub__, PyObjectCallbackConverter)
    }
}

trait PyNumberMulProtocolImpl {
    fn nb_multiply() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberMulProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberMulProtocolImpl for T where T: PyNumberMulProtocol {
    fn nb_multiply() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberMulProtocol, T::__mul__, PyObjectCallbackConverter)
    }
}

trait PyNumberMatmulProtocolImpl {
    fn nb_matrix_multiply() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberMatmulProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_matrix_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberMatmulProtocolImpl for T where T: PyNumberMatmulProtocol {
    fn nb_matrix_multiply() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberMatmulProtocol, T::__matmul__, PyObjectCallbackConverter)
    }
}

trait PyNumberTruedivProtocolImpl {
    fn nb_true_divide() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberTruedivProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_true_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberTruedivProtocolImpl for T where T: PyNumberTruedivProtocol {
    fn nb_true_divide() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberTruedivProtocol, T::__truediv__, PyObjectCallbackConverter)
    }
}

trait PyNumberFloordivProtocolImpl {
    fn nb_floor_divide() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberFloordivProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_floor_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberFloordivProtocolImpl for T where T: PyNumberFloordivProtocol {
    fn nb_floor_divide() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberFloordivProtocol, T::__floordiv__, PyObjectCallbackConverter)
    }
}

trait PyNumberModProtocolImpl {
    fn nb_remainder() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberModProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_remainder() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberModProtocolImpl for T where T: PyNumberModProtocol {
    fn nb_remainder() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberModProtocol, T::__mod__, PyObjectCallbackConverter)
    }
}

trait PyNumberDivmodProtocolImpl {
    fn nb_divmod() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberDivmodProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_divmod() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberDivmodProtocolImpl for T where T: PyNumberDivmodProtocol {
    fn nb_divmod() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberDivmodProtocol, T::__divmod__, PyObjectCallbackConverter)
    }
}

trait PyNumberPowProtocolImpl {
    fn nb_power() -> Option<ffi::ternaryfunc>;
}
impl<T> PyNumberPowProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_power() -> Option<ffi::ternaryfunc> {None}
}
impl<T> PyNumberPowProtocolImpl for T where T: PyNumberPowProtocol {
    fn nb_power() -> Option<ffi::ternaryfunc> {
        py_ternary_func!(PyNumberPowProtocol, T::__pow__, PyObjectCallbackConverter)
    }
}

trait PyNumberLShiftProtocolImpl {
    fn nb_lshift() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberLShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_lshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberLShiftProtocolImpl for T where T: PyNumberLShiftProtocol {
    fn nb_lshift() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberLShiftProtocol, T::__lshift__, PyObjectCallbackConverter)
    }
}

trait PyNumberRShiftProtocolImpl {
    fn nb_rshift() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberRShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_rshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberRShiftProtocolImpl for T where T: PyNumberRShiftProtocol {
    fn nb_rshift() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberRShiftProtocol, T::__rshift__, PyObjectCallbackConverter)
    }
}


trait PyNumberAndProtocolImpl {
    fn nb_and() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberAndProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_and() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberAndProtocolImpl for T where T: PyNumberAndProtocol {
    fn nb_and() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberAndProtocol, T::__and__, PyObjectCallbackConverter)
    }
}

trait PyNumberXorProtocolImpl {
    fn nb_xor() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberXorProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_xor() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberXorProtocolImpl for T where T: PyNumberXorProtocol {
    fn nb_xor() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberXorProtocol, T::__xor__, PyObjectCallbackConverter)
    }
}

trait PyNumberOrProtocolImpl {
    fn nb_or() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberOrProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_or() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberOrProtocolImpl for T where T: PyNumberOrProtocol {
    fn nb_or() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberOrProtocol, T::__or__, PyObjectCallbackConverter)
    }
}


trait PyNumberIAddProtocolImpl {
    fn nb_inplace_add() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIAddProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_add() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIAddProtocolImpl for T where T: PyNumberIAddProtocol {
    fn nb_inplace_add() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIAddProtocol, T::__iadd__, PyObjectCallbackConverter)
    }
}

trait PyNumberISubProtocolImpl {
    fn nb_inplace_subtract() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberISubProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_subtract() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberISubProtocolImpl for T where T: PyNumberISubProtocol {
    fn nb_inplace_subtract() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberISubProtocol, T::__isub__, PyObjectCallbackConverter)
    }
}

trait PyNumberIMulProtocolImpl {
    fn nb_inplace_multiply() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIMulProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIMulProtocolImpl for T where T: PyNumberIMulProtocol {
    fn nb_inplace_multiply() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIMulProtocol, T::__imul__, PyObjectCallbackConverter)
    }
}

trait PyNumberIMatmulProtocolImpl {
    fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIMatmulProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIMatmulProtocolImpl for T where T: PyNumberIMatmulProtocol {
    fn nb_inplace_matrix_multiply() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIMatmulProtocol, T::__imatmul__, PyObjectCallbackConverter)
    }
}

trait PyNumberITruedivProtocolImpl {
    fn nb_inplace_true_divide() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberITruedivProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_true_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberITruedivProtocolImpl for T where T: PyNumberITruedivProtocol {
    fn nb_inplace_true_divide() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberITruedivProtocol, T::__itruediv__, PyObjectCallbackConverter)
    }
}

trait PyNumberIFloordivProtocolImpl {
    fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIFloordivProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIFloordivProtocolImpl for T where T: PyNumberIFloordivProtocol {
    fn nb_inplace_floor_divide() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIFloordivProtocol, T::__ifloordiv__, PyObjectCallbackConverter)
    }
}

trait PyNumberIModProtocolImpl {
    fn nb_inplace_remainder() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIModProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_remainder() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIModProtocolImpl for T where T: PyNumberIModProtocol {
    fn nb_inplace_remainder() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIModProtocol, T::__imod__, PyObjectCallbackConverter)
    }
}

trait PyNumberIPowProtocolImpl {
    fn nb_inplace_power() -> Option<ffi::ternaryfunc>;
}
impl<T> PyNumberIPowProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_power() -> Option<ffi::ternaryfunc> {None}
}
impl<T> PyNumberIPowProtocolImpl for T where T: PyNumberIPowProtocol {
    fn nb_inplace_power() -> Option<ffi::ternaryfunc> {
        py_ternary_func!(PyNumberIPowProtocol, T::__ipow__, PyObjectCallbackConverter)
    }
}

trait PyNumberILShiftProtocolImpl {
    fn nb_inplace_lshift() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberILShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_lshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberILShiftProtocolImpl for T where T: PyNumberILShiftProtocol {
    fn nb_inplace_lshift() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberILShiftProtocol, T::__ilshift__, PyObjectCallbackConverter)
    }
}

trait PyNumberIRShiftProtocolImpl {
    fn nb_inplace_rshift() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIRShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_rshift() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIRShiftProtocolImpl for T where T: PyNumberIRShiftProtocol {
    fn nb_inplace_rshift() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIRShiftProtocol, T::__irshift__, PyObjectCallbackConverter)
    }
}


trait PyNumberIAndProtocolImpl {
    fn nb_inplace_and() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIAndProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_and() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIAndProtocolImpl for T where T: PyNumberIAndProtocol {
    fn nb_inplace_and() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIAndProtocol, T::__iand__, PyObjectCallbackConverter)
    }
}

trait PyNumberIXorProtocolImpl {
    fn nb_inplace_xor() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIXorProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_xor() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIXorProtocolImpl for T where T: PyNumberIXorProtocol {
    fn nb_inplace_xor() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIXorProtocol, T::__ixor__, PyObjectCallbackConverter)
    }
}

trait PyNumberIOrProtocolImpl {
    fn nb_inplace_or() -> Option<ffi::binaryfunc>;
}
impl<T> PyNumberIOrProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_inplace_or() -> Option<ffi::binaryfunc> {None}
}
impl<T> PyNumberIOrProtocolImpl for T where T: PyNumberIOrProtocol {
    fn nb_inplace_or() -> Option<ffi::binaryfunc> {
        py_binary_func_!(PyNumberIOrProtocol, T::__ior__, PyObjectCallbackConverter)
    }
}


trait PyNumberRAddProtocolImpl {
    fn __radd__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRAddProtocolImpl for T where T: PyNumberProtocol {
    default fn __radd__() -> Option<PyMethodDef> {None}
}

trait PyNumberRSubProtocolImpl {
    fn __rsub__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRSubProtocolImpl for T where T: PyNumberProtocol {
    default fn __rsub__() -> Option<PyMethodDef> {None}
}

trait PyNumberRMulProtocolImpl {
    fn __rmul__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRMulProtocolImpl for T where T: PyNumberProtocol {
    default fn __rmul__() -> Option<PyMethodDef> {None}
}

trait PyNumberRMatmulProtocolImpl {
    fn __rmatmul__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRMatmulProtocolImpl for T where T: PyNumberProtocol {
    default fn __rmatmul__() -> Option<PyMethodDef> {None}
}

trait PyNumberRTruedivProtocolImpl {
    fn __rtruediv__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRTruedivProtocolImpl for T where T: PyNumberProtocol {
    default fn __rtruediv__() -> Option<PyMethodDef> {None}
}

trait PyNumberRFloordivProtocolImpl {
    fn __rfloordiv__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRFloordivProtocolImpl for T where T: PyNumberProtocol {
    default fn __rfloordiv__() -> Option<PyMethodDef> {None}
}

trait PyNumberRModProtocolImpl {
    fn __rmod__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRModProtocolImpl for T where T: PyNumberProtocol {
    default fn __rmod__() -> Option<PyMethodDef> {None}
}

trait PyNumberRDivmodProtocolImpl {
    fn __rdivmod__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRDivmodProtocolImpl for T where T: PyNumberProtocol {
    default fn __rdivmod__() -> Option<PyMethodDef> {None}
}

trait PyNumberRPowProtocolImpl {
    fn __rpow__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRPowProtocolImpl for T where T: PyNumberProtocol {
    default fn __rpow__() -> Option<PyMethodDef> {None}
}

trait PyNumberRLShiftProtocolImpl {
    fn __rlshift__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRLShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn __rlshift__() -> Option<PyMethodDef> {None}
}

trait PyNumberRRShiftProtocolImpl {
    fn __rrshift__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRRShiftProtocolImpl for T where T: PyNumberProtocol {
    default fn __rrshift__() -> Option<PyMethodDef> {None}
}


trait PyNumberRAndProtocolImpl {
    fn __rand__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRAndProtocolImpl for T where T: PyNumberProtocol {
    default fn __rand__() -> Option<PyMethodDef> {None}
}

trait PyNumberRXorProtocolImpl {
    fn __rxor__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRXorProtocolImpl for T where T: PyNumberProtocol {
    default fn __rxor__() -> Option<PyMethodDef> {None}
}

trait PyNumberROrProtocolImpl {
    fn __ror__() -> Option<PyMethodDef>;
}
impl<T> PyNumberROrProtocolImpl for T where T: PyNumberProtocol {
    default fn __ror__() -> Option<PyMethodDef> {None}
}

trait PyNumberNegProtocolImpl {
    fn nb_negative() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberNegProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_negative() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberNegProtocolImpl for T
    where T: PyNumberNegProtocol
{
    #[inline]
    fn nb_negative() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberNegProtocol, T::__neg__, PyObjectCallbackConverter)
    }
}

trait PyNumberPosProtocolImpl {
    fn nb_positive() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberPosProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_positive() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberPosProtocolImpl for T where T: PyNumberPosProtocol
{
    fn nb_positive() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberPosProtocol, T::__pos__, PyObjectCallbackConverter)
    }
}

trait PyNumberAbsProtocolImpl {
    fn nb_absolute() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberAbsProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_absolute() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberAbsProtocolImpl for T where T: PyNumberAbsProtocol
{
    fn nb_absolute() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberAbsProtocol, T::__abs__, PyObjectCallbackConverter)
    }
}

trait PyNumberInvertProtocolImpl {
    fn nb_invert() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberInvertProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_invert() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberInvertProtocolImpl for T where T: PyNumberInvertProtocol
{
    fn nb_invert() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberInvertProtocol, T::__invert__, PyObjectCallbackConverter)
    }
}

trait PyNumberIntProtocolImpl {
    fn nb_int() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberIntProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_int() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberIntProtocolImpl for T where T: PyNumberIntProtocol {
    fn nb_int() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberIntProtocol, T::__int__, PyObjectCallbackConverter)
    }
}

trait PyNumberFloatProtocolImpl {
    fn nb_float() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberFloatProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_float() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberFloatProtocolImpl for T where T: PyNumberFloatProtocol {
    fn nb_float() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberFloatProtocol, T::__float__, PyObjectCallbackConverter)
    }
}

trait PyNumberIndexProtocolImpl {
    fn nb_index() -> Option<ffi::unaryfunc>;
}
impl<T> PyNumberIndexProtocolImpl for T where T: PyNumberProtocol {
    default fn nb_index() -> Option<ffi::unaryfunc> {None}
}
impl<T> PyNumberIndexProtocolImpl for T where T: PyNumberIndexProtocol {
    fn nb_index() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyNumberIndexProtocol, T::__index__, PyObjectCallbackConverter)
    }
}

trait PyNumberComplexProtocolImpl {
    fn __complex__() -> Option<PyMethodDef>;
}
impl<T> PyNumberComplexProtocolImpl for T where T: PyNumberProtocol {
    default fn __complex__() -> Option<PyMethodDef> {None}
}

trait PyNumberRoundProtocolImpl {
    fn __round__() -> Option<PyMethodDef>;
}
impl<T> PyNumberRoundProtocolImpl for T where T: PyNumberProtocol {
    default fn __round__() -> Option<PyMethodDef> {None}
}
