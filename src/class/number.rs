// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol
use crate::err::PyErr;
use crate::{callback::IntoPyCallbackOutput, derive_utils::TryFromPyCell};
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

    fn __radd__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRAddProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rsub__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRSubProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmul__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRMulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmatmul__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRMatmulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rtruediv__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRTruedivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rfloordiv__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRFloordivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rmod__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRModProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rdivmod__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRDivmodProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rpow__(
        slf: Self::Receiver,
        other: Self::Other,
        modulo: Option<Self::Modulo>,
    ) -> Self::Result
    where
        Self: PyNumberRPowProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rlshift__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRLShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rrshift__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRRShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rand__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRAndProtocol<'p>,
    {
        unimplemented!()
    }
    fn __rxor__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberRXorProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ror__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberROrProtocol<'p>,
    {
        unimplemented!()
    }

    fn __iadd__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIAddProtocol<'p>,
    {
        unimplemented!()
    }
    fn __isub__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberISubProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imul__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIMulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imatmul__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIMatmulProtocol<'p>,
    {
        unimplemented!()
    }
    fn __itruediv__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberITruedivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ifloordiv__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIFloordivProtocol<'p>,
    {
        unimplemented!()
    }
    fn __imod__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIModProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ipow__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIPowProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ilshift__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberILShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __irshift__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIRShiftProtocol<'p>,
    {
        unimplemented!()
    }
    fn __iand__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIAndProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ixor__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIXorProtocol<'p>,
    {
        unimplemented!()
    }
    fn __ior__(slf: Self::Receiver, other: Self::Other) -> Self::Result
    where
        Self: PyNumberIOrProtocol<'p>,
    {
        unimplemented!()
    }

    // Unary arithmetic
    fn __neg__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberNegProtocol<'p>,
    {
        unimplemented!()
    }
    fn __pos__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberPosProtocol<'p>,
    {
        unimplemented!()
    }
    fn __abs__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberAbsProtocol<'p>,
    {
        unimplemented!()
    }
    fn __invert__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberInvertProtocol<'p>,
    {
        unimplemented!()
    }
    fn __complex__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberComplexProtocol<'p>,
    {
        unimplemented!()
    }
    fn __int__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberIntProtocol<'p>,
    {
        unimplemented!()
    }
    fn __float__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberFloatProtocol<'p>,
    {
        unimplemented!()
    }
    fn __index__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyNumberIndexProtocol<'p>,
    {
        unimplemented!()
    }
    fn __round__(slf: Self::Receiver, ndigits: Option<Self::NDigits>) -> Self::Result
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
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRSubProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRMulProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRTruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRModProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRPowProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Modulo: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberRLShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberRRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRAndProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRXorProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberROrProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIAddProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberISubProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMulProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIMatmulProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberITruedivProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIFloordivProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIModProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIDivmodProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIPowProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberILShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

#[allow(clippy::upper_case_acronyms)]
pub trait PyNumberIRShiftProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIAndProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIXorProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberIOrProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Other: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<()>;
}

pub trait PyNumberNegProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberPosProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberAbsProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberInvertProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberComplexProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIntProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberFloatProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberRoundProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type NDigits: FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyNumberIndexProtocol<'p>: PyNumberProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
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
py_binary_num_func!(mod_, PyNumberModProtocol, T::__mod__);
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

                let borrow =
                    <<T as PyNumberRPowProtocol>::Receiver as TryFromPyCell<_>>::try_from_pycell(
                        slf,
                    )
                    .map_err(|e| e.into())?;

                T::__rpow__(borrow, arg, modulo).convert(py)
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
        let borrow =
            <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf).map_err(|e| e.into())?;
        T::__rpow__(borrow, arg, modulo).convert(py)
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
py_binary_inplace_func!(iadd, PyNumberIAddProtocol, T::__iadd__);
py_binary_inplace_func!(isub, PyNumberISubProtocol, T::__isub__);
py_binary_inplace_func!(imul, PyNumberIMulProtocol, T::__imul__);
py_binary_inplace_func!(imod, PyNumberIModProtocol, T::__imod__);

#[doc(hidden)]
pub unsafe extern "C" fn ipow<T>(
    slf: *mut ffi::PyObject,
    other: *mut ffi::PyObject,
    _modulo: *mut ffi::PyObject,
) -> *mut ffi::PyObject
where
    T: for<'p> PyNumberIPowProtocol<'p>,
{
    // NOTE: Somehow __ipow__ causes SIGSEGV in Python < 3.8 when we extract,
    // so we ignore it. It's the same as what CPython does.
    crate::callback_body!(py, {
        let slf_cell = py.from_borrowed_ptr::<crate::PyCell<T>>(slf);
        let other = py.from_borrowed_ptr::<crate::PyAny>(other);
        let borrow =
            <T::Receiver as TryFromPyCell<_>>::try_from_pycell(slf_cell).map_err(|e| e.into())?;
        T::__ipow__(borrow, extract_or_return_not_implemented!(other)).convert(py)?;
        ffi::Py_INCREF(slf);
        Ok::<_, PyErr>(slf)
    })
}

py_binary_inplace_func!(ilshift, PyNumberILShiftProtocol, T::__ilshift__);
py_binary_inplace_func!(irshift, PyNumberIRShiftProtocol, T::__irshift__);
py_binary_inplace_func!(iand, PyNumberIAndProtocol, T::__iand__);
py_binary_inplace_func!(ixor, PyNumberIXorProtocol, T::__ixor__);
py_binary_inplace_func!(ior, PyNumberIOrProtocol, T::__ior__);
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
py_binary_inplace_func!(ifloordiv, PyNumberIFloordivProtocol, T::__ifloordiv__);
py_binary_inplace_func!(itruediv, PyNumberITruedivProtocol, T::__itruediv__);
py_unary_func!(index, PyNumberIndexProtocol, T::__index__);
py_binary_fallback_num_func!(
    matmul_rmatmul,
    T,
    PyNumberMatmulProtocol::__matmul__,
    PyNumberRMatmulProtocol::__rmatmul__
);
py_binary_num_func!(matmul, PyNumberMatmulProtocol, T::__matmul__);
py_binary_reversed_num_func!(rmatmul, PyNumberRMatmulProtocol, T::__rmatmul__);
py_binary_inplace_func!(imatmul, PyNumberIMatmulProtocol, T::__imatmul__);
