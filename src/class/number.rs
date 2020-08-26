// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol

use crate::err::PyErr;
use crate::callback::IntoPyCallbackOutput;
use crate::{ffi, FromPyObject, PyClass, PyObject};

/// Number interface
#[allow(unused_variables)]
pub trait PyNumberProtocol<'p>: PyClass {
    fn __add__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberAddProtocol<'p>,
    {
        unimplemented!()
    }
    fn __sub__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberSubProtocol<'p>,
    {
        unimplemented!()
    }
    fn __mul__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberMulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __matmul__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberMatmulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __truediv__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberTruedivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __floordiv__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberFloordivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __mod__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberModProtocol<'p>,
    {
        unimplemented!()
    }
    fn __divmod__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberDivmodProtocol<'p>,
    {
        unimplemented!()
    }
    fn __pow__(lhs: Self::Left, rhs: Self::Right, modulo: Option<Self::Modulo>) -> Self::Result
    where
        Self: PyNumberPowProtocol<'p>,
    {
        unimplemented!()
    }
    fn __lshift__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberLShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rshift__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberRShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __and__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberAndProtocol<'p>,
    {
        unimplemented!()
    }
    fn __xor__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberXorProtocol<'p>,
    {
        unimplemented!()
    }
    fn __or__(lhs: Self::Left, rhs: Self::Right) -> Self::Result
    where
        Self: PyNumberOrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __radd__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRAddProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rsub__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRSubProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmul__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRMulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmatmul__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRMatmulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rtruediv__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRTruedivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rfloordiv__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRFloordivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmod__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRModProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rdivmod__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRDivmodProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rpow__(&'p self, other: Self::Other, modulo: Option<Self::Modulo>) -> Self::Result
    where
        Self: PyNumberRPowProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rlshift__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRLShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rrshift__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRRShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rand__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRAndProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rxor__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRXorProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ror__(&'p self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberROrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __iadd__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIAddProtocol<'p>,
    {
        unimplemented!()
    }
    fn __isub__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberISubProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imul__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIMulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imatmul__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIMatmulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __itruediv__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberITruedivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ifloordiv__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIFloordivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imod__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIModProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ipow__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIPowProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ilshift__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberILShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __irshift__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIRShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __iand__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIAndProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ixor__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIXorProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ior__(&'p mut self, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIOrProtocol<'p>,
    {
        unimplemented!()
    }

    // Unary arithmetic
    fn __neg__(&'p self) -> Self::Result
    where
        Self: PyNumberNegProtocol<'p>,
    {
        unimplemented!()
    }
    fn __pos__(&'p self) -> Self::Result
    where
        Self: PyNumberPosProtocol<'p>,
    {
        unimplemented!()
    }
    fn __abs__(&'p self) -> Self::Result
    where
        Self: PyNumberAbsProtocol<'p>,
    {
        unimplemented!()
    }
    fn __invert__(&'p self) -> Self::Result
    where
        Self: PyNumberInvertProtocol<'p>,
    {
        unimplemented!()
    }
    fn __complex__(&'p self) -> Self::Result
    where
        Self: PyNumberComplexProtocol<'p>,
    {
        unimplemented!()
    }
    fn __int__(&'p self) -> Self::Result
    where
        Self: PyNumberIntProtocol<'p>,
    {
        unimplemented!()
    }
    fn __float__(&'p self) -> Self::Result
    where
        Self: PyNumberFloatProtocol<'p>,
    {
        unimplemented!()
    }
    fn __index__(&'p self) -> Self::Result
    where
        Self: PyNumberIndexProtocol<'p>,
    {
        unimplemented!()
    }
    fn __round__(&'p self, ndigits: Option<Self::NDigits>) -> Self::Result
    where
        Self: PyNumberRoundProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyNumberAddProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberSubProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberMulProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberTruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberModProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberPowProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberLShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberAndProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberXorProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberOrProtocol<'p>: PyNumberProtocol<'p> {
    type Left: FromPyObject<'p>;
    type Right: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRAddProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRSubProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRMulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRTruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRModProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRPowProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRLShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRAndProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRXorProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberROrProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIAddProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberISubProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberITruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIModProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIPowProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberILShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIAndProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIXorProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIOrProtocol<'p>: PyNumberProtocol<'p> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberNegProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberPosProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberAbsProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberInvertProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberComplexProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIntProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberFloatProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRoundProtocol<'p>: PyNumberProtocol<'p> {
    type NDigits: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIndexProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

#[doc(hidden)]
impl ffi::PyNumberMethods {
    pub(crate) fn from_nb_bool(nb_bool: ffi::inquiry) -> *mut Self {
        let mut nm = ffi::PyNumberMethods_INIT;
        nm.nb_bool = Some(nb_bool);
        Box::into_raw(Box::new(nm))
    }
    pub fn set_add_radd<T>(&mut self)
    where
        T: for<'p> PyNumberAddProtocol<'p> + for<'p> PyNumberRAddProtocol<'p>,
    {
        self.nb_add = py_binary_fallback_num_func!(
            T,
            PyNumberAddProtocol::__add__,
            PyNumberRAddProtocol::__radd__
        );
    }
    pub fn set_add<T>(&mut self)
    where
        T: for<'p> PyNumberAddProtocol<'p>,
    {
        self.nb_add = py_binary_num_func!(PyNumberAddProtocol, T::__add__);
    }
    pub fn set_radd<T>(&mut self)
    where
        T: for<'p> PyNumberRAddProtocol<'p>,
    {
        self.nb_add = py_binary_reversed_num_func!(PyNumberRAddProtocol, T::__radd__);
    }
    pub fn set_sub_rsub<T>(&mut self)
    where
        T: for<'p> PyNumberSubProtocol<'p> + for<'p> PyNumberRSubProtocol<'p>,
    {
        self.nb_subtract = py_binary_fallback_num_func!(
            T,
            PyNumberSubProtocol::__sub__,
            PyNumberRSubProtocol::__rsub__
        );
    }
    pub fn set_sub<T>(&mut self)
    where
        T: for<'p> PyNumberSubProtocol<'p>,
    {
        self.nb_subtract = py_binary_num_func!(PyNumberSubProtocol, T::__sub__);
    }
    pub fn set_rsub<T>(&mut self)
    where
        T: for<'p> PyNumberRSubProtocol<'p>,
    {
        self.nb_subtract = py_binary_reversed_num_func!(PyNumberRSubProtocol, T::__rsub__);
    }
    pub fn set_mul_rmul<T>(&mut self)
    where
        T: for<'p> PyNumberMulProtocol<'p> + for<'p> PyNumberRMulProtocol<'p>,
    {
        self.nb_multiply = py_binary_fallback_num_func!(
            T,
            PyNumberMulProtocol::__mul__,
            PyNumberRMulProtocol::__rmul__
        );
    }
    pub fn set_mul<T>(&mut self)
    where
        T: for<'p> PyNumberMulProtocol<'p>,
    {
        self.nb_multiply = py_binary_num_func!(PyNumberMulProtocol, T::__mul__);
    }
    pub fn set_rmul<T>(&mut self)
    where
        T: for<'p> PyNumberRMulProtocol<'p>,
    {
        self.nb_multiply = py_binary_reversed_num_func!(PyNumberRMulProtocol, T::__rmul__);
    }
    pub fn set_mod<T>(&mut self)
    where
        T: for<'p> PyNumberModProtocol<'p>,
    {
        self.nb_remainder = py_binary_num_func!(PyNumberModProtocol, T::__mod__);
    }
    pub fn set_divmod_rdivmod<T>(&mut self)
    where
        T: for<'p> PyNumberDivmodProtocol<'p> + for<'p> PyNumberRDivmodProtocol<'p>,
    {
        self.nb_divmod = py_binary_fallback_num_func!(
            T,
            PyNumberDivmodProtocol::__divmod__,
            PyNumberRDivmodProtocol::__rdivmod__
        );
    }
    pub fn set_divmod<T>(&mut self)
    where
        T: for<'p> PyNumberDivmodProtocol<'p>,
    {
        self.nb_divmod = py_binary_num_func!(PyNumberDivmodProtocol, T::__divmod__);
    }
    pub fn set_rdivmod<T>(&mut self)
    where
        T: for<'p> PyNumberRDivmodProtocol<'p>,
    {
        self.nb_divmod = py_binary_reversed_num_func!(PyNumberRDivmodProtocol, T::__rdivmod__);
    }
    pub fn set_pow_rpow<T>(&mut self)
    where
        T: for<'p> PyNumberPowProtocol<'p> + for<'p> PyNumberRPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_pow_and_rpow<T>(
            lhs: *mut crate::ffi::PyObject,
            rhs: *mut crate::ffi::PyObject,
            modulo: *mut crate::ffi::PyObject,
        ) -> *mut crate::ffi::PyObject
        where
            T: for<'p> PyNumberPowProtocol<'p> + for<'p> PyNumberRPowProtocol<'p>,
        {
            crate::callback_body!(py, {
                let lhs = py.from_borrowed_ptr::<crate::PyAny>(lhs);
                let rhs = py.from_borrowed_ptr::<crate::PyAny>(rhs);
                let modulo = py.from_borrowed_ptr::<crate::PyAny>(modulo);
                // First, try __pow__
                match (lhs.extract(), rhs.extract(), modulo.extract()) {
                    (Ok(l), Ok(r), Ok(m)) => T::__pow__(l, r, m).convert(py),
                    _ => {
                        // Then try __rpow__
                        let slf: &crate::PyCell<T> = extract_or_return_not_implemented!(rhs);
                        let arg = extract_or_return_not_implemented!(lhs);
                        let modulo = extract_or_return_not_implemented!(modulo);
                        slf.try_borrow()?.__rpow__(arg, modulo).convert(py)
                    }
                }
            })
        }
        self.nb_power = Some(wrap_pow_and_rpow::<T>);
    }
    pub fn set_pow<T>(&mut self)
    where
        T: for<'p> PyNumberPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_pow<T>(
            lhs: *mut crate::ffi::PyObject,
            rhs: *mut crate::ffi::PyObject,
            modulo: *mut crate::ffi::PyObject,
        ) -> *mut crate::ffi::PyObject
        where
            T: for<'p> PyNumberPowProtocol<'p>,
        {
            crate::callback_body!(py, {
                let lhs = extract_or_return_not_implemented!(py, lhs);
                let rhs = extract_or_return_not_implemented!(py, rhs);
                let modulo = extract_or_return_not_implemented!(py, modulo);
                T::__pow__(lhs, rhs, modulo).convert(py)
            })
        }
        self.nb_power = Some(wrap_pow::<T>);
    }
    pub fn set_rpow<T>(&mut self)
    where
        T: for<'p> PyNumberRPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_rpow<T>(
            arg: *mut crate::ffi::PyObject,
            slf: *mut crate::ffi::PyObject,
            modulo: *mut crate::ffi::PyObject,
        ) -> *mut crate::ffi::PyObject
        where
            T: for<'p> PyNumberRPowProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf: &crate::PyCell<T> = extract_or_return_not_implemented!(py, slf);
                let arg = extract_or_return_not_implemented!(py, arg);
                let modulo = extract_or_return_not_implemented!(py, modulo);
                slf.try_borrow()?.__rpow__(arg, modulo).convert(py)
            })
        }
        self.nb_power = Some(wrap_rpow::<T>);
    }
    pub fn set_neg<T>(&mut self)
    where
        T: for<'p> PyNumberNegProtocol<'p>,
    {
        self.nb_negative = py_unary_func!(PyNumberNegProtocol, T::__neg__);
    }
    pub fn set_pos<T>(&mut self)
    where
        T: for<'p> PyNumberPosProtocol<'p>,
    {
        self.nb_positive = py_unary_func!(PyNumberPosProtocol, T::__pos__);
    }
    pub fn set_abs<T>(&mut self)
    where
        T: for<'p> PyNumberAbsProtocol<'p>,
    {
        self.nb_absolute = py_unary_func!(PyNumberAbsProtocol, T::__abs__);
    }
    pub fn set_invert<T>(&mut self)
    where
        T: for<'p> PyNumberInvertProtocol<'p>,
    {
        self.nb_invert = py_unary_func!(PyNumberInvertProtocol, T::__invert__);
    }
    pub fn set_lshift_rlshift<T>(&mut self)
    where
        T: for<'p> PyNumberLShiftProtocol<'p> + for<'p> PyNumberRLShiftProtocol<'p>,
    {
        self.nb_lshift = py_binary_fallback_num_func!(
            T,
            PyNumberLShiftProtocol::__lshift__,
            PyNumberRLShiftProtocol::__rlshift__
        );
    }
    pub fn set_lshift<T>(&mut self)
    where
        T: for<'p> PyNumberLShiftProtocol<'p>,
    {
        self.nb_lshift = py_binary_num_func!(PyNumberLShiftProtocol, T::__lshift__);
    }
    pub fn set_rlshift<T>(&mut self)
    where
        T: for<'p> PyNumberRLShiftProtocol<'p>,
    {
        self.nb_lshift = py_binary_reversed_num_func!(PyNumberRLShiftProtocol, T::__rlshift__);
    }
    pub fn set_rshift_rrshift<T>(&mut self)
    where
        T: for<'p> PyNumberRShiftProtocol<'p> + for<'p> PyNumberRRShiftProtocol<'p>,
    {
        self.nb_rshift = py_binary_fallback_num_func!(
            T,
            PyNumberRShiftProtocol::__rshift__,
            PyNumberRRShiftProtocol::__rrshift__
        );
    }
    pub fn set_rshift<T>(&mut self)
    where
        T: for<'p> PyNumberRShiftProtocol<'p>,
    {
        self.nb_rshift = py_binary_num_func!(PyNumberRShiftProtocol, T::__rshift__);
    }
    pub fn set_rrshift<T>(&mut self)
    where
        T: for<'p> PyNumberRRShiftProtocol<'p>,
    {
        self.nb_rshift = py_binary_reversed_num_func!(PyNumberRRShiftProtocol, T::__rrshift__);
    }
    pub fn set_and_rand<T>(&mut self)
    where
        T: for<'p> PyNumberAndProtocol<'p> + for<'p> PyNumberRAndProtocol<'p>,
    {
        self.nb_and = py_binary_fallback_num_func!(
            T,
            PyNumberAndProtocol::__and__,
            PyNumberRAndProtocol::__rand__
        );
    }
    pub fn set_and<T>(&mut self)
    where
        T: for<'p> PyNumberAndProtocol<'p>,
    {
        self.nb_and = py_binary_num_func!(PyNumberAndProtocol, T::__and__);
    }
    pub fn set_rand<T>(&mut self)
    where
        T: for<'p> PyNumberRAndProtocol<'p>,
    {
        self.nb_and = py_binary_reversed_num_func!(PyNumberRAndProtocol, T::__rand__);
    }
    pub fn set_xor_rxor<T>(&mut self)
    where
        T: for<'p> PyNumberXorProtocol<'p> + for<'p> PyNumberRXorProtocol<'p>,
    {
        self.nb_xor = py_binary_fallback_num_func!(
            T,
            PyNumberXorProtocol::__xor__,
            PyNumberRXorProtocol::__rxor__
        );
    }
    pub fn set_xor<T>(&mut self)
    where
        T: for<'p> PyNumberXorProtocol<'p>,
    {
        self.nb_xor = py_binary_num_func!(PyNumberXorProtocol, T::__xor__);
    }
    pub fn set_rxor<T>(&mut self)
    where
        T: for<'p> PyNumberRXorProtocol<'p>,
    {
        self.nb_xor = py_binary_reversed_num_func!(PyNumberRXorProtocol, T::__rxor__);
    }
    pub fn set_or_ror<T>(&mut self)
    where
        T: for<'p> PyNumberOrProtocol<'p> + for<'p> PyNumberROrProtocol<'p>,
    {
        self.nb_or = py_binary_fallback_num_func!(
            T,
            PyNumberOrProtocol::__or__,
            PyNumberROrProtocol::__ror__
        );
    }
    pub fn set_or<T>(&mut self)
    where
        T: for<'p> PyNumberOrProtocol<'p>,
    {
        self.nb_or = py_binary_num_func!(PyNumberOrProtocol, T::__or__);
    }
    pub fn set_ror<T>(&mut self)
    where
        T: for<'p> PyNumberROrProtocol<'p>,
    {
        self.nb_or = py_binary_reversed_num_func!(PyNumberROrProtocol, T::__ror__);
    }
    pub fn set_int<T>(&mut self)
    where
        T: for<'p> PyNumberIntProtocol<'p>,
    {
        self.nb_int = py_unary_func!(PyNumberIntProtocol, T::__int__);
    }
    pub fn set_float<T>(&mut self)
    where
        T: for<'p> PyNumberFloatProtocol<'p>,
    {
        self.nb_float = py_unary_func!(PyNumberFloatProtocol, T::__float__);
    }
    pub fn set_iadd<T>(&mut self)
    where
        T: for<'p> PyNumberIAddProtocol<'p>,
    {
        self.nb_inplace_add = py_binary_self_func!(PyNumberIAddProtocol, T::__iadd__);
    }
    pub fn set_isub<T>(&mut self)
    where
        T: for<'p> PyNumberISubProtocol<'p>,
    {
        self.nb_inplace_subtract = py_binary_self_func!(PyNumberISubProtocol, T::__isub__);
    }
    pub fn set_imul<T>(&mut self)
    where
        T: for<'p> PyNumberIMulProtocol<'p>,
    {
        self.nb_inplace_multiply = py_binary_self_func!(PyNumberIMulProtocol, T::__imul__);
    }
    pub fn set_imod<T>(&mut self)
    where
        T: for<'p> PyNumberIModProtocol<'p>,
    {
        self.nb_inplace_remainder = py_binary_self_func!(PyNumberIModProtocol, T::__imod__);
    }
    pub fn set_ipow<T>(&mut self)
    where
        T: for<'p> PyNumberIPowProtocol<'p>,
    {
        // NOTE: Somehow __ipow__ causes SIGSEGV in Python < 3.8 when we extract,
        // so we ignore it. It's the same as what CPython does.
        unsafe extern "C" fn wrap_ipow<T>(
            slf: *mut crate::ffi::PyObject,
            other: *mut crate::ffi::PyObject,
            _modulo: *mut crate::ffi::PyObject,
        ) -> *mut crate::ffi::PyObject
        where
            T: for<'p> PyNumberIPowProtocol<'p>,
        {
            crate::callback_body!(py, {
                let slf_cell = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
                let other = py.from_borrowed_ptr::<crate::PyAny>(other);
                call_operator_mut!(py, slf_cell, __ipow__, other).convert(py)?;
                ffi::Py_INCREF(slf);
                Ok::<_, PyErr>(slf)
            })
        }
        self.nb_inplace_power = Some(wrap_ipow::<T>);
    }
    pub fn set_ilshift<T>(&mut self)
    where
        T: for<'p> PyNumberILShiftProtocol<'p>,
    {
        self.nb_inplace_lshift = py_binary_self_func!(PyNumberILShiftProtocol, T::__ilshift__);
    }
    pub fn set_irshift<T>(&mut self)
    where
        T: for<'p> PyNumberIRShiftProtocol<'p>,
    {
        self.nb_inplace_rshift = py_binary_self_func!(PyNumberIRShiftProtocol, T::__irshift__);
    }
    pub fn set_iand<T>(&mut self)
    where
        T: for<'p> PyNumberIAndProtocol<'p>,
    {
        self.nb_inplace_and = py_binary_self_func!(PyNumberIAndProtocol, T::__iand__);
    }
    pub fn set_ixor<T>(&mut self)
    where
        T: for<'p> PyNumberIXorProtocol<'p>,
    {
        self.nb_inplace_xor = py_binary_self_func!(PyNumberIXorProtocol, T::__ixor__);
    }
    pub fn set_ior<T>(&mut self)
    where
        T: for<'p> PyNumberIOrProtocol<'p>,
    {
        self.nb_inplace_or = py_binary_self_func!(PyNumberIOrProtocol, T::__ior__);
    }
    pub fn set_floordiv_rfloordiv<T>(&mut self)
    where
        T: for<'p> PyNumberFloordivProtocol<'p> + for<'p> PyNumberRFloordivProtocol<'p>,
    {
        self.nb_floor_divide = py_binary_fallback_num_func!(
            T,
            PyNumberFloordivProtocol::__floordiv__,
            PyNumberRFloordivProtocol::__rfloordiv__
        );
    }
    pub fn set_floordiv<T>(&mut self)
    where
        T: for<'p> PyNumberFloordivProtocol<'p>,
    {
        self.nb_floor_divide = py_binary_num_func!(PyNumberFloordivProtocol, T::__floordiv__);
    }
    pub fn set_rfloordiv<T>(&mut self)
    where
        T: for<'p> PyNumberRFloordivProtocol<'p>,
    {
        self.nb_floor_divide =
            py_binary_reversed_num_func!(PyNumberRFloordivProtocol, T::__rfloordiv__);
    }
    pub fn set_truediv_rtruediv<T>(&mut self)
    where
        T: for<'p> PyNumberTruedivProtocol<'p> + for<'p> PyNumberRTruedivProtocol<'p>,
    {
        self.nb_true_divide = py_binary_fallback_num_func!(
            T,
            PyNumberTruedivProtocol::__truediv__,
            PyNumberRTruedivProtocol::__rtruediv__
        );
    }
    pub fn set_truediv<T>(&mut self)
    where
        T: for<'p> PyNumberTruedivProtocol<'p>,
    {
        self.nb_true_divide = py_binary_num_func!(PyNumberTruedivProtocol, T::__truediv__);
    }
    pub fn set_rtruediv<T>(&mut self)
    where
        T: for<'p> PyNumberRTruedivProtocol<'p>,
    {
        self.nb_true_divide =
            py_binary_reversed_num_func!(PyNumberRTruedivProtocol, T::__rtruediv__);
    }
    pub fn set_ifloordiv<T>(&mut self)
    where
        T: for<'p> PyNumberIFloordivProtocol<'p>,
    {
        self.nb_inplace_floor_divide =
            py_binary_self_func!(PyNumberIFloordivProtocol, T::__ifloordiv__);
    }
    pub fn set_itruediv<T>(&mut self)
    where
        T: for<'p> PyNumberITruedivProtocol<'p>,
    {
        self.nb_inplace_true_divide =
            py_binary_self_func!(PyNumberITruedivProtocol, T::__itruediv__);
    }
    pub fn set_index<T>(&mut self)
    where
        T: for<'p> PyNumberIndexProtocol<'p>,
    {
        self.nb_index = py_unary_func!(PyNumberIndexProtocol, T::__index__);
    }
    pub fn set_matmul_rmatmul<T>(&mut self)
    where
        T: for<'p> PyNumberMatmulProtocol<'p> + for<'p> PyNumberRMatmulProtocol<'p>,
    {
        self.nb_matrix_multiply = py_binary_fallback_num_func!(
            T,
            PyNumberMatmulProtocol::__matmul__,
            PyNumberRMatmulProtocol::__rmatmul__
        );
    }
    pub fn set_matmul<T>(&mut self)
    where
        T: for<'p> PyNumberMatmulProtocol<'p>,
    {
        self.nb_matrix_multiply = py_binary_num_func!(PyNumberMatmulProtocol, T::__matmul__);
    }
    pub fn set_rmatmul<T>(&mut self)
    where
        T: for<'p> PyNumberRMatmulProtocol<'p>,
    {
        self.nb_matrix_multiply =
            py_binary_reversed_num_func!(PyNumberRMatmulProtocol, T::__rmatmul__);
    }
    pub fn set_imatmul<T>(&mut self)
    where
        T: for<'p> PyNumberIMatmulProtocol<'p>,
    {
        self.nb_inplace_matrix_multiply =
            py_binary_self_func!(PyNumberIMatmulProtocol, T::__imatmul__);
    }
}
