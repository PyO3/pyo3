#![allow(deprecated)]
// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol
use crate::callback::IntoPyCallbackOutput;
use crate::err::PyErr;
use crate::pyclass::boolean_struct::False;
use crate::{ffi, FromPyObject, PyClass, PyObject};

/// Number interface
#[allow(unused_variables)]
#[deprecated(since = "0.16.0", note = "prefer `#[pymethods]` to `#[pyproto]`")]
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
    fn __ipow__(&'p mut self, other: Self::Other, modulo: Option<Self::Modulo>) -> Self::Result
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

pub trait PyNumberIAddProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberISubProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMulProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMatmulProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberITruedivProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIFloordivProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIModProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIDivmodProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIPowProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
    // See https://bugs.python.org/issue36379
    type Modulo: FromPyObject<'p>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberILShiftProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberIRShiftProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIAndProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIXorProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIOrProtocol<'p>: PyNumberProtocol<'p> + PyClass<Frozen = False> {
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

pub trait PyNumberIntProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberFloatProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIndexProtocol<'p>: PyNumberProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

py_binary_fallback_num_func!(
    add_radd,
    T,
    PyNumberAddProtocol::__add__,
    PyNumberRAddProtocol::__radd__
);
py_binary_num_func!(add, PyNumberAddProtocol, T::__add__);
py_binary_reversed_num_func!(radd, PyNumberRAddProtocol, T::__radd__);
py_binary_fallback_num_func!(
    sub_rsub,
    T,
    PyNumberSubProtocol::__sub__,
    PyNumberRSubProtocol::__rsub__
);
py_binary_num_func!(sub, PyNumberSubProtocol, T::__sub__);
py_binary_reversed_num_func!(rsub, PyNumberRSubProtocol, T::__rsub__);
py_binary_fallback_num_func!(
    mul_rmul,
    T,
    PyNumberMulProtocol::__mul__,
    PyNumberRMulProtocol::__rmul__
);
py_binary_num_func!(mul, PyNumberMulProtocol, T::__mul__);
py_binary_reversed_num_func!(rmul, PyNumberRMulProtocol, T::__rmul__);
py_binary_fallback_num_func!(
    mod_rmod,
    T,
    PyNumberModProtocol::__mod__,
    PyNumberRModProtocol::__rmod__
);
py_binary_num_func!(mod_, PyNumberModProtocol, T::__mod__);
py_binary_reversed_num_func!(rmod, PyNumberRModProtocol, T::__rmod__);
py_binary_fallback_num_func!(
    divmod_rdivmod,
    T,
    PyNumberDivmodProtocol::__divmod__,
    PyNumberRDivmodProtocol::__rdivmod__
);
py_binary_num_func!(divmod, PyNumberDivmodProtocol, T::__divmod__);
py_binary_reversed_num_func!(rdivmod, PyNumberRDivmodProtocol, T::__rdivmod__);

#[doc(hidden)]
pub unsafe extern "C" fn pow_rpow<T>(
    lhs: *mut ffi::PyObject,
    rhs: *mut ffi::PyObject,
    modulo: *mut ffi::PyObject,
) -> *mut ffi::PyObject
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

#[doc(hidden)]
pub unsafe extern "C" fn pow<T>(
    lhs: *mut ffi::PyObject,
    rhs: *mut ffi::PyObject,
    modulo: *mut ffi::PyObject,
) -> *mut ffi::PyObject
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

#[doc(hidden)]
pub unsafe extern "C" fn rpow<T>(
    arg: *mut ffi::PyObject,
    slf: *mut ffi::PyObject,
    modulo: *mut ffi::PyObject,
) -> *mut ffi::PyObject
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

py_unary_func!(neg, PyNumberNegProtocol, T::__neg__);
py_unary_func!(pos, PyNumberPosProtocol, T::__pos__);
py_unary_func!(abs, PyNumberAbsProtocol, T::__abs__);
py_unary_func!(invert, PyNumberInvertProtocol, T::__invert__);
py_binary_fallback_num_func!(
    lshift_rlshift,
    T,
    PyNumberLShiftProtocol::__lshift__,
    PyNumberRLShiftProtocol::__rlshift__
);
py_binary_num_func!(lshift, PyNumberLShiftProtocol, T::__lshift__);
py_binary_reversed_num_func!(rlshift, PyNumberRLShiftProtocol, T::__rlshift__);
py_binary_fallback_num_func!(
    rshift_rrshift,
    T,
    PyNumberRShiftProtocol::__rshift__,
    PyNumberRRShiftProtocol::__rrshift__
);
py_binary_num_func!(rshift, PyNumberRShiftProtocol, T::__rshift__);
py_binary_reversed_num_func!(rrshift, PyNumberRRShiftProtocol, T::__rrshift__);
py_binary_fallback_num_func!(
    and_rand,
    T,
    PyNumberAndProtocol::__and__,
    PyNumberRAndProtocol::__rand__
);
py_binary_num_func!(and, PyNumberAndProtocol, T::__and__);
py_binary_reversed_num_func!(rand, PyNumberRAndProtocol, T::__rand__);
py_binary_fallback_num_func!(
    xor_rxor,
    T,
    PyNumberXorProtocol::__xor__,
    PyNumberRXorProtocol::__rxor__
);
py_binary_num_func!(xor, PyNumberXorProtocol, T::__xor__);
py_binary_reversed_num_func!(rxor, PyNumberRXorProtocol, T::__rxor__);
py_binary_fallback_num_func!(
    or_ror,
    T,
    PyNumberOrProtocol::__or__,
    PyNumberROrProtocol::__ror__
);
py_binary_num_func!(or, PyNumberOrProtocol, T::__or__);
py_binary_reversed_num_func!(ror, PyNumberROrProtocol, T::__ror__);
py_unary_func!(int, PyNumberIntProtocol, T::__int__);
py_unary_func!(float, PyNumberFloatProtocol, T::__float__);
py_binary_self_func!(iadd, PyNumberIAddProtocol, T::__iadd__);
py_binary_self_func!(isub, PyNumberISubProtocol, T::__isub__);
py_binary_self_func!(imul, PyNumberIMulProtocol, T::__imul__);
py_binary_self_func!(imod, PyNumberIModProtocol, T::__imod__);

#[doc(hidden)]
pub unsafe extern "C" fn ipow<T>(
    slf: *mut ffi::PyObject,
    other: *mut ffi::PyObject,
    modulo: crate::impl_::pymethods::IPowModulo,
) -> *mut ffi::PyObject
where
    T: for<'p> PyNumberIPowProtocol<'p>,
{
    crate::callback_body!(py, {
        let slf_cell = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
        let other = py.from_borrowed_ptr::<crate::PyAny>(other);
        slf_cell
            .try_borrow_mut()?
            .__ipow__(
                extract_or_return_not_implemented!(other),
                match modulo.to_borrowed_any(py).extract() {
                    Ok(value) => value,
                    Err(_) => {
                        let res = crate::ffi::Py_NotImplemented();
                        crate::ffi::Py_INCREF(res);
                        return Ok(res);
                    }
                },
            )
            .convert(py)?;
        ffi::Py_INCREF(slf);
        Ok::<_, PyErr>(slf)
    })
}

py_binary_self_func!(ilshift, PyNumberILShiftProtocol, T::__ilshift__);
py_binary_self_func!(irshift, PyNumberIRShiftProtocol, T::__irshift__);
py_binary_self_func!(iand, PyNumberIAndProtocol, T::__iand__);
py_binary_self_func!(ixor, PyNumberIXorProtocol, T::__ixor__);
py_binary_self_func!(ior, PyNumberIOrProtocol, T::__ior__);
py_binary_fallback_num_func!(
    floordiv_rfloordiv,
    T,
    PyNumberFloordivProtocol::__floordiv__,
    PyNumberRFloordivProtocol::__rfloordiv__
);
py_binary_num_func!(floordiv, PyNumberFloordivProtocol, T::__floordiv__);
py_binary_reversed_num_func!(rfloordiv, PyNumberRFloordivProtocol, T::__rfloordiv__);
py_binary_fallback_num_func!(
    truediv_rtruediv,
    T,
    PyNumberTruedivProtocol::__truediv__,
    PyNumberRTruedivProtocol::__rtruediv__
);
py_binary_num_func!(truediv, PyNumberTruedivProtocol, T::__truediv__);
py_binary_reversed_num_func!(rtruediv, PyNumberRTruedivProtocol, T::__rtruediv__);
py_binary_self_func!(ifloordiv, PyNumberIFloordivProtocol, T::__ifloordiv__);
py_binary_self_func!(itruediv, PyNumberITruedivProtocol, T::__itruediv__);
py_unary_func!(index, PyNumberIndexProtocol, T::__index__);
py_binary_fallback_num_func!(
    matmul_rmatmul,
    T,
    PyNumberMatmulProtocol::__matmul__,
    PyNumberRMatmulProtocol::__rmatmul__
);
py_binary_num_func!(matmul, PyNumberMatmulProtocol, T::__matmul__);
py_binary_reversed_num_func!(rmatmul, PyNumberRMatmulProtocol, T::__rmatmul__);
py_binary_self_func!(imatmul, PyNumberIMatmulProtocol, T::__imatmul__);
