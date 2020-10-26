// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol
use super::proto_methods::TypedSlot;
use crate::callback::IntoPyCallbackOutput;
use crate::err::PyErr;
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

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyNumberSlots {
    fn get_add_radd() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberAddProtocol<'p> + for<'p> PyNumberRAddProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_add,
            py_binary_fallback_num_func!(
                Self,
                PyNumberAddProtocol::__add__,
                PyNumberRAddProtocol::__radd__
            ),
        )
    }

    fn get_add() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberAddProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_add,
            py_binary_num_func!(PyNumberAddProtocol, Self::__add__),
        )
    }

    fn get_radd() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRAddProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_add,
            py_binary_reversed_num_func!(PyNumberRAddProtocol, Self::__radd__),
        )
    }

    fn get_sub_rsub() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberSubProtocol<'p> + for<'p> PyNumberRSubProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_subtract,
            py_binary_fallback_num_func!(
                Self,
                PyNumberSubProtocol::__sub__,
                PyNumberRSubProtocol::__rsub__
            ),
        )
    }

    fn get_sub() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberSubProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_subtract,
            py_binary_num_func!(PyNumberSubProtocol, Self::__sub__),
        )
    }

    fn get_rsub() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRSubProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_subtract,
            py_binary_reversed_num_func!(PyNumberRSubProtocol, Self::__rsub__),
        )
    }

    fn get_mul_rmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberMulProtocol<'p> + for<'p> PyNumberRMulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_multiply,
            py_binary_fallback_num_func!(
                Self,
                PyNumberMulProtocol::__mul__,
                PyNumberRMulProtocol::__rmul__
            ),
        )
    }

    fn get_mul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberMulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_multiply,
            py_binary_num_func!(PyNumberMulProtocol, Self::__mul__),
        )
    }

    fn get_rmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRMulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_multiply,
            py_binary_reversed_num_func!(PyNumberRMulProtocol, Self::__rmul__),
        )
    }

    fn get_mod() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberModProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_remainder,
            py_binary_num_func!(PyNumberModProtocol, Self::__mod__),
        )
    }

    fn get_divmod_rdivmod() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberDivmodProtocol<'p> + for<'p> PyNumberRDivmodProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_divmod,
            py_binary_fallback_num_func!(
                Self,
                PyNumberDivmodProtocol::__divmod__,
                PyNumberRDivmodProtocol::__rdivmod__
            ),
        )
    }

    fn get_divmod() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberDivmodProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_divmod,
            py_binary_num_func!(PyNumberDivmodProtocol, Self::__divmod__),
        )
    }

    fn get_rdivmod() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRDivmodProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_divmod,
            py_binary_reversed_num_func!(PyNumberRDivmodProtocol, Self::__rdivmod__),
        )
    }

    fn get_pow_rpow() -> TypedSlot<ffi::ternaryfunc>
    where
        Self: for<'p> PyNumberPowProtocol<'p> + for<'p> PyNumberRPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_pow_and_rpow<T>(
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

        TypedSlot(ffi::Py_nb_power, wrap_pow_and_rpow::<Self>)
    }

    fn get_pow() -> TypedSlot<ffi::ternaryfunc>
    where
        Self: for<'p> PyNumberPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_pow<T>(
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

        TypedSlot(ffi::Py_nb_power, wrap_pow::<Self>)
    }

    fn get_rpow() -> TypedSlot<ffi::ternaryfunc>
    where
        Self: for<'p> PyNumberRPowProtocol<'p>,
    {
        unsafe extern "C" fn wrap_rpow<T>(
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

        TypedSlot(ffi::Py_nb_power, wrap_rpow::<Self>)
    }

    fn get_neg() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberNegProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_negative,
            py_unary_func!(PyNumberNegProtocol, Self::__neg__),
        )
    }

    fn get_pos() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberPosProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_positive,
            py_unary_func!(PyNumberPosProtocol, Self::__pos__),
        )
    }

    fn get_abs() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberAbsProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_absolute,
            py_unary_func!(PyNumberAbsProtocol, Self::__abs__),
        )
    }

    fn get_invert() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberInvertProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_invert,
            py_unary_func!(PyNumberInvertProtocol, Self::__invert__),
        )
    }

    fn get_lshift_rlshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberLShiftProtocol<'p> + for<'p> PyNumberRLShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_lshift,
            py_binary_fallback_num_func!(
                Self,
                PyNumberLShiftProtocol::__lshift__,
                PyNumberRLShiftProtocol::__rlshift__
            ),
        )
    }

    fn get_lshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberLShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_lshift,
            py_binary_num_func!(PyNumberLShiftProtocol, Self::__lshift__),
        )
    }

    fn get_rlshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRLShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_lshift,
            py_binary_reversed_num_func!(PyNumberRLShiftProtocol, Self::__rlshift__),
        )
    }

    fn get_rshift_rrshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRShiftProtocol<'p> + for<'p> PyNumberRRShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_rshift,
            py_binary_fallback_num_func!(
                Self,
                PyNumberRShiftProtocol::__rshift__,
                PyNumberRRShiftProtocol::__rrshift__
            ),
        )
    }

    fn get_rshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_rshift,
            py_binary_num_func!(PyNumberRShiftProtocol, Self::__rshift__),
        )
    }

    fn get_rrshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRRShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_rshift,
            py_binary_reversed_num_func!(PyNumberRRShiftProtocol, Self::__rrshift__),
        )
    }

    fn get_and_rand() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberAndProtocol<'p> + for<'p> PyNumberRAndProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_and,
            py_binary_fallback_num_func!(
                Self,
                PyNumberAndProtocol::__and__,
                PyNumberRAndProtocol::__rand__
            ),
        )
    }

    fn get_and() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberAndProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_and,
            py_binary_num_func!(PyNumberAndProtocol, Self::__and__),
        )
    }

    fn get_rand() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRAndProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_and,
            py_binary_reversed_num_func!(PyNumberRAndProtocol, Self::__rand__),
        )
    }

    fn get_xor_rxor() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberXorProtocol<'p> + for<'p> PyNumberRXorProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_xor,
            py_binary_fallback_num_func!(
                Self,
                PyNumberXorProtocol::__xor__,
                PyNumberRXorProtocol::__rxor__
            ),
        )
    }

    fn get_xor() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberXorProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_xor,
            py_binary_num_func!(PyNumberXorProtocol, Self::__xor__),
        )
    }

    fn get_rxor() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRXorProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_xor,
            py_binary_reversed_num_func!(PyNumberRXorProtocol, Self::__rxor__),
        )
    }

    fn get_or_ror() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberOrProtocol<'p> + for<'p> PyNumberROrProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_or,
            py_binary_fallback_num_func!(
                Self,
                PyNumberOrProtocol::__or__,
                PyNumberROrProtocol::__ror__
            ),
        )
    }

    fn get_or() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberOrProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_or,
            py_binary_num_func!(PyNumberOrProtocol, Self::__or__),
        )
    }

    fn get_ror() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberROrProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_or,
            py_binary_reversed_num_func!(PyNumberROrProtocol, Self::__ror__),
        )
    }

    fn get_int() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberIntProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_int,
            py_unary_func!(PyNumberIntProtocol, Self::__int__),
        )
    }

    fn get_float() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberFloatProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_float,
            py_unary_func!(PyNumberFloatProtocol, Self::__float__),
        )
    }

    fn get_iadd() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIAddProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_add,
            py_binary_self_func!(PyNumberIAddProtocol, Self::__iadd__),
        )
    }

    fn get_isub() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberISubProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_subtract,
            py_binary_self_func!(PyNumberISubProtocol, Self::__isub__),
        )
    }

    fn get_imul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIMulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_multiply,
            py_binary_self_func!(PyNumberIMulProtocol, Self::__imul__),
        )
    }

    fn get_imod() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIModProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_remainder,
            py_binary_self_func!(PyNumberIModProtocol, Self::__imod__),
        )
    }

    fn get_ipow() -> TypedSlot<ffi::ternaryfunc>
    where
        Self: for<'p> PyNumberIPowProtocol<'p>,
    {
        // NOTE: Somehow __ipow__ causes SIGSEGV in Python < 3.8 when we extract,
        // so we ignore it. It's the same as what CPython does.
        unsafe extern "C" fn wrap_ipow<T>(
            slf: *mut ffi::PyObject,
            other: *mut ffi::PyObject,
            _modulo: *mut ffi::PyObject,
        ) -> *mut ffi::PyObject
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

        TypedSlot(ffi::Py_nb_inplace_power, wrap_ipow::<Self>)
    }

    fn get_ilshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberILShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_lshift,
            py_binary_self_func!(PyNumberILShiftProtocol, Self::__ilshift__),
        )
    }

    fn get_irshift() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIRShiftProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_rshift,
            py_binary_self_func!(PyNumberIRShiftProtocol, Self::__irshift__),
        )
    }

    fn get_iand() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIAndProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_and,
            py_binary_self_func!(PyNumberIAndProtocol, Self::__iand__),
        )
    }

    fn get_ixor() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIXorProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_xor,
            py_binary_self_func!(PyNumberIXorProtocol, Self::__ixor__),
        )
    }

    fn get_ior() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIOrProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_or,
            py_binary_self_func!(PyNumberIOrProtocol, Self::__ior__),
        )
    }

    fn get_floordiv_rfloordiv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberFloordivProtocol<'p> + for<'p> PyNumberRFloordivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_floor_divide,
            py_binary_fallback_num_func!(
                Self,
                PyNumberFloordivProtocol::__floordiv__,
                PyNumberRFloordivProtocol::__rfloordiv__
            ),
        )
    }

    fn get_floordiv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberFloordivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_floor_divide,
            py_binary_num_func!(PyNumberFloordivProtocol, Self::__floordiv__),
        )
    }

    fn get_rfloordiv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRFloordivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_floor_divide,
            py_binary_reversed_num_func!(PyNumberRFloordivProtocol, Self::__rfloordiv__),
        )
    }

    fn get_truediv_rtruediv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberTruedivProtocol<'p> + for<'p> PyNumberRTruedivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_true_divide,
            py_binary_fallback_num_func!(
                Self,
                PyNumberTruedivProtocol::__truediv__,
                PyNumberRTruedivProtocol::__rtruediv__
            ),
        )
    }

    fn get_truediv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberTruedivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_true_divide,
            py_binary_num_func!(PyNumberTruedivProtocol, Self::__truediv__),
        )
    }

    fn get_rtruediv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRTruedivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_true_divide,
            py_binary_reversed_num_func!(PyNumberRTruedivProtocol, Self::__rtruediv__),
        )
    }

    fn get_ifloordiv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIFloordivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_floor_divide,
            py_binary_self_func!(PyNumberIFloordivProtocol, Self::__ifloordiv__),
        )
    }

    fn get_itruediv() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberITruedivProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_true_divide,
            py_binary_self_func!(PyNumberITruedivProtocol, Self::__itruediv__),
        )
    }

    fn get_index() -> TypedSlot<ffi::unaryfunc>
    where
        Self: for<'p> PyNumberIndexProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_index,
            py_unary_func!(PyNumberIndexProtocol, Self::__index__),
        )
    }

    fn get_matmul_rmatmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberMatmulProtocol<'p> + for<'p> PyNumberRMatmulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_matrix_multiply,
            py_binary_fallback_num_func!(
                Self,
                PyNumberMatmulProtocol::__matmul__,
                PyNumberRMatmulProtocol::__rmatmul__
            ),
        )
    }

    fn get_matmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberMatmulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_matrix_multiply,
            py_binary_num_func!(PyNumberMatmulProtocol, Self::__matmul__),
        )
    }

    fn get_rmatmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberRMatmulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_matrix_multiply,
            py_binary_reversed_num_func!(PyNumberRMatmulProtocol, Self::__rmatmul__),
        )
    }

    fn get_imatmul() -> TypedSlot<ffi::binaryfunc>
    where
        Self: for<'p> PyNumberIMatmulProtocol<'p>,
    {
        TypedSlot(
            ffi::Py_nb_inplace_matrix_multiply,
            py_binary_self_func!(PyNumberIMatmulProtocol, Self::__imatmul__),
        )
    }
}

impl<'p, T> PyNumberSlots for T where T: PyNumberProtocol<'p> {}
