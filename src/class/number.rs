// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Number Interface
//! Trait and support implementation for implementing number protocol
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
    fn get_add_radd() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberAddProtocol<'p> + for<'p> PyNumberRAddProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_add,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberAddProtocol::__add__,
                PyNumberRAddProtocol::__radd__
            ) as _,
        }
    }

    fn get_add() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberAddProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_add,
            pfunc: py_binary_num_func!(PyNumberAddProtocol, Self::__add__) as _,
        }
    }

    fn get_radd() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRAddProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_add,
            pfunc: py_binary_reversed_num_func!(PyNumberRAddProtocol, Self::__radd__) as _,
        }
    }

    fn get_sub_rsub() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberSubProtocol<'p> + for<'p> PyNumberRSubProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_subtract,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberSubProtocol::__sub__,
                PyNumberRSubProtocol::__rsub__
            ) as _,
        }
    }

    fn get_sub() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberSubProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_subtract,
            pfunc: py_binary_num_func!(PyNumberSubProtocol, Self::__sub__) as _,
        }
    }

    fn get_rsub() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRSubProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_subtract,
            pfunc: py_binary_reversed_num_func!(PyNumberRSubProtocol, Self::__rsub__) as _,
        }
    }

    fn get_mul_rmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberMulProtocol<'p> + for<'p> PyNumberRMulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_multiply,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberMulProtocol::__mul__,
                PyNumberRMulProtocol::__rmul__
            ) as _,
        }
    }

    fn get_mul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberMulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_multiply,
            pfunc: py_binary_num_func!(PyNumberMulProtocol, Self::__mul__) as _,
        }
    }

    fn get_rmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRMulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_multiply,
            pfunc: py_binary_reversed_num_func!(PyNumberRMulProtocol, Self::__rmul__) as _,
        }
    }

    fn get_mod() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberModProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_remainder,
            pfunc: py_binary_num_func!(PyNumberModProtocol, Self::__mod__) as _,
        }
    }

    fn get_divmod_rdivmod() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberDivmodProtocol<'p> + for<'p> PyNumberRDivmodProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_divmod,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberDivmodProtocol::__divmod__,
                PyNumberRDivmodProtocol::__rdivmod__
            ) as _,
        }
    }

    fn get_divmod() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberDivmodProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_divmod,
            pfunc: py_binary_num_func!(PyNumberDivmodProtocol, Self::__divmod__) as _,
        }
    }

    fn get_rdivmod() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRDivmodProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_divmod,
            pfunc: py_binary_reversed_num_func!(PyNumberRDivmodProtocol, Self::__rdivmod__) as _,
        }
    }

    fn get_pow_rpow() -> ffi::PyType_Slot
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

        ffi::PyType_Slot {
            slot: ffi::Py_nb_power,
            pfunc: wrap_pow_and_rpow::<Self> as _,
        }
    }

    fn get_pow() -> ffi::PyType_Slot
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

        ffi::PyType_Slot {
            slot: ffi::Py_nb_power,
            pfunc: wrap_pow::<Self> as _,
        }
    }

    fn get_rpow() -> ffi::PyType_Slot
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

        ffi::PyType_Slot {
            slot: ffi::Py_nb_power,
            pfunc: wrap_rpow::<Self> as _,
        }
    }

    fn get_neg() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberNegProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_negative,
            pfunc: py_unary_func!(PyNumberNegProtocol, Self::__neg__) as _,
        }
    }

    fn get_pos<T>() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberPosProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_absolute,
            pfunc: py_unary_func!(PyNumberPosProtocol, Self::__pos__) as _,
        }
    }

    fn get_abs<T>() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberAbsProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_absolute,
            pfunc: py_unary_func!(PyNumberAbsProtocol, Self::__abs__) as _,
        }
    }

    fn get_invert<T>() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberInvertProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_invert,
            pfunc: py_unary_func!(PyNumberInvertProtocol, Self::__invert__) as _,
        }
    }

    fn get_lshift_rlshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberLShiftProtocol<'p> + for<'p> PyNumberRLShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_lshift,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberLShiftProtocol::__lshift__,
                PyNumberRLShiftProtocol::__rlshift__
            ) as _,
        }
    }

    fn get_lshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberLShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_lshift,
            pfunc: py_binary_num_func!(PyNumberLShiftProtocol, Self::__lshift__) as _,
        }
    }

    fn get_rlshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRLShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_lshift,
            pfunc: py_binary_reversed_num_func!(PyNumberRLShiftProtocol, Self::__rlshift__) as _,
        }
    }

    fn get_rshift_rrshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRShiftProtocol<'p> + for<'p> PyNumberRRShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_rshift,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberRShiftProtocol::__rshift__,
                PyNumberRRShiftProtocol::__rrshift__
            ) as _,
        }
    }

    fn get_rshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_rshift,
            pfunc: py_binary_num_func!(PyNumberRShiftProtocol, Self::__rshift__) as _,
        }
    }

    fn get_rrshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRRShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_rshift,
            pfunc: py_binary_reversed_num_func!(PyNumberRRShiftProtocol, Self::__rrshift__) as _,
        }
    }

    fn get_and_rand() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberAndProtocol<'p> + for<'p> PyNumberRAndProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_and,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberAndProtocol::__and__,
                PyNumberRAndProtocol::__rand__
            ) as _,
        }
    }

    fn get_and() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberAndProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_and,
            pfunc: py_binary_num_func!(PyNumberAndProtocol, Self::__and__) as _,
        }
    }

    fn get_rand() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRAndProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_and,
            pfunc: py_binary_reversed_num_func!(PyNumberRAndProtocol, Self::__rand__) as _,
        }
    }

    fn get_xor_rxor() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberXorProtocol<'p> + for<'p> PyNumberRXorProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_xor,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberXorProtocol::__xor__,
                PyNumberRXorProtocol::__rxor__
            ) as _,
        }
    }

    fn get_xor() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberXorProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_xor,
            pfunc: py_binary_num_func!(PyNumberXorProtocol, Self::__xor__) as _,
        }
    }

    fn get_rxor() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRXorProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_xor,
            pfunc: py_binary_reversed_num_func!(PyNumberRXorProtocol, Self::__rxor__) as _,
        }
    }

    fn get_or_ror() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberOrProtocol<'p> + for<'p> PyNumberROrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_or,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberOrProtocol::__or__,
                PyNumberROrProtocol::__ror__
            ) as _,
        }
    }

    fn get_or() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberOrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_or,
            pfunc: py_binary_num_func!(PyNumberOrProtocol, Self::__or__) as _,
        }
    }

    fn get_ror() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberROrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_or,
            pfunc: py_binary_reversed_num_func!(PyNumberROrProtocol, Self::__ror__) as _,
        }
    }

    fn get_int() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIntProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_int,
            pfunc: py_unary_func!(PyNumberIntProtocol, Self::__int__) as _,
        }
    }

    fn get_float() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberFloatProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_float,
            pfunc: py_unary_func!(PyNumberFloatProtocol, Self::__float__) as _,
        }
    }

    fn get_iadd() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIAddProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_add,
            pfunc: py_binary_self_func!(PyNumberIAddProtocol, Self::__iadd__) as _,
        }
    }

    fn get_isub() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberISubProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_subtract,
            pfunc: py_binary_self_func!(PyNumberISubProtocol, Self::__isub__) as _,
        }
    }

    fn get_imul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIMulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_multiply,
            pfunc: py_binary_self_func!(PyNumberIMulProtocol, Self::__imul__) as _,
        }
    }

    fn get_imod() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIModProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_remainder,
            pfunc: py_binary_self_func!(PyNumberIModProtocol, Self::__imod__) as _,
        }
    }

    fn get_ipow() -> ffi::PyType_Slot
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

        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_power,
            pfunc: wrap_ipow::<Self> as _,
        }
    }

    fn get_ilshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberILShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_lshift,
            pfunc: py_binary_self_func!(PyNumberILShiftProtocol, Self::__ilshift__) as _,
        }
    }

    fn get_irshift() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIRShiftProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_rshift,
            pfunc: py_binary_self_func!(PyNumberIRShiftProtocol, Self::__irshift__) as _,
        }
    }

    fn get_iand() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIAndProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_and,
            pfunc: py_binary_self_func!(PyNumberIAndProtocol, Self::__iand__) as _,
        }
    }

    fn get_ixor() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIXorProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_xor,
            pfunc: py_binary_self_func!(PyNumberIXorProtocol, Self::__ixor__) as _,
        }
    }

    fn get_ior() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIOrProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_or,
            pfunc: py_binary_self_func!(PyNumberIOrProtocol, Self::__ior__) as _,
        }
    }

    fn get_floordiv_rfloordiv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberFloordivProtocol<'p> + for<'p> PyNumberRFloordivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_floor_divide,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberFloordivProtocol::__floordiv__,
                PyNumberRFloordivProtocol::__rfloordiv__
            ) as _,
        }
    }

    fn get_floordiv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberFloordivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_floor_divide,
            pfunc: py_binary_num_func!(PyNumberFloordivProtocol, Self::__floordiv__) as _,
        }
    }

    fn get_rfloordiv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRFloordivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_floor_divide,
            pfunc: py_binary_reversed_num_func!(PyNumberRFloordivProtocol, Self::__rfloordiv__)
                as _,
        }
    }

    fn get_truediv_rtruediv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberTruedivProtocol<'p> + for<'p> PyNumberRTruedivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_true_divide,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberTruedivProtocol::__truediv__,
                PyNumberRTruedivProtocol::__rtruediv__
            ) as _,
        }
    }

    fn get_truediv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberTruedivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_true_divide,
            pfunc: py_binary_num_func!(PyNumberTruedivProtocol, Self::__truediv__) as _,
        }
    }

    fn get_rtruediv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRTruedivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_true_divide,
            pfunc: py_binary_reversed_num_func!(PyNumberRTruedivProtocol, Self::__rtruediv__) as _,
        }
    }

    fn get_ifloordiv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIFloordivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_floor_divide,
            pfunc: py_binary_self_func!(PyNumberIFloordivProtocol, Self::__ifloordiv__) as _,
        }
    }

    fn get_itruediv() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberITruedivProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_true_divide,
            pfunc: py_binary_self_func!(PyNumberITruedivProtocol, Self::__itruediv__) as _,
        }
    }

    fn get_index() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIndexProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_index,
            pfunc: py_unary_func!(PyNumberIndexProtocol, Self::__index__) as _,
        }
    }

    fn get_matmul_rmatmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberMatmulProtocol<'p> + for<'p> PyNumberRMatmulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_matrix_multiply,
            pfunc: py_binary_fallback_num_func!(
                Self,
                PyNumberMatmulProtocol::__matmul__,
                PyNumberRMatmulProtocol::__rmatmul__
            ) as _,
        }
    }

    fn get_matmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberMatmulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_matrix_multiply,
            pfunc: py_binary_num_func!(PyNumberMatmulProtocol, Self::__matmul__) as _,
        }
    }

    fn get_rmatmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberRMatmulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_matrix_multiply,
            pfunc: py_binary_reversed_num_func!(PyNumberRMatmulProtocol, Self::__rmatmul__) as _,
        }
    }

    fn get_imatmul() -> ffi::PyType_Slot
    where
        Self: for<'p> PyNumberIMatmulProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_nb_inplace_matrix_multiply,
            pfunc: py_binary_self_func!(PyNumberIMatmulProtocol, Self::__imatmul__) as _,
        }
    }
}

impl<'p, T> PyNumberSlots for T where T: PyNumberProtocol<'p> {}
