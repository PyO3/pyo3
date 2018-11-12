#![feature(specialization)]

extern crate pyo3;

use pyo3::class::*;
use pyo3::prelude::*;
use pyo3::types::PyObjectRef;

#[macro_use]
mod common;

#[pyclass]
struct UnaryArithmetic {}

#[pyproto]
impl PyNumberProtocol for UnaryArithmetic {
    fn __neg__(&self) -> PyResult<&'static str> {
        Ok("neg")
    }

    fn __pos__(&self) -> PyResult<&'static str> {
        Ok("pos")
    }

    fn __abs__(&self) -> PyResult<&'static str> {
        Ok("abs")
    }

    fn __invert__(&self) -> PyResult<&'static str> {
        Ok("invert")
    }
}

#[test]
fn unary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|_| UnaryArithmetic {}).unwrap();
    py_run!(py, c, "assert -c == 'neg'");
    py_run!(py, c, "assert +c == 'pos'");
    py_run!(py, c, "assert abs(c) == 'abs'");
    py_run!(py, c, "assert ~c == 'invert'");
}

#[pyclass]
struct BinaryArithmetic {}

#[pyproto]
impl PyObjectProtocol for BinaryArithmetic {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[pyclass]
struct InPlaceOperations {
    value: u32,
}

#[pyproto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("IPO({:?})", self.value))
    }
}

#[pyproto]
impl PyNumberProtocol for InPlaceOperations {
    fn __iadd__(&mut self, other: u32) -> PyResult<()> {
        self.value += other;
        Ok(())
    }

    fn __isub__(&mut self, other: u32) -> PyResult<()> {
        self.value -= other;
        Ok(())
    }

    fn __imul__(&mut self, other: u32) -> PyResult<()> {
        self.value *= other;
        Ok(())
    }

    fn __ilshift__(&mut self, other: u32) -> PyResult<()> {
        self.value <<= other;
        Ok(())
    }

    fn __irshift__(&mut self, other: u32) -> PyResult<()> {
        self.value >>= other;
        Ok(())
    }

    fn __iand__(&mut self, other: u32) -> PyResult<()> {
        self.value &= other;
        Ok(())
    }

    fn __ixor__(&mut self, other: u32) -> PyResult<()> {
        self.value ^= other;
        Ok(())
    }

    fn __ior__(&mut self, other: u32) -> PyResult<()> {
        self.value |= other;
        Ok(())
    }
}

#[test]
fn inplace_operations() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let init = |value, code| {
        let c = py.init(|_| InPlaceOperations { value }).unwrap();
        py_run!(py, c, code);
    };

    init(0, "d = c; c += 1; assert repr(c) == repr(d) == 'IPO(1)'");
    init(10, "d = c; c -= 1; assert repr(c) == repr(d) == 'IPO(9)'");
    init(3, "d = c; c *= 3; assert repr(c) == repr(d) == 'IPO(9)'");
    init(3, "d = c; c <<= 2; assert repr(c) == repr(d) == 'IPO(12)'");
    init(12, "d = c; c >>= 2; assert repr(c) == repr(d) == 'IPO(3)'");
    init(12, "d = c; c &= 10; assert repr(c) == repr(d) == 'IPO(8)'");
    init(12, "d = c; c |= 3; assert repr(c) == repr(d) == 'IPO(15)'");
    init(12, "d = c; c ^= 5; assert repr(c) == repr(d) == 'IPO(9)'");
}

#[pyproto]
impl PyNumberProtocol for BinaryArithmetic {
    fn __add__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", lhs, rhs))
    }

    fn __sub__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", lhs, rhs))
    }

    fn __mul__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} * {:?}", lhs, rhs))
    }

    fn __lshift__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} << {:?}", lhs, rhs))
    }

    fn __rshift__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} >> {:?}", lhs, rhs))
    }

    fn __and__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} & {:?}", lhs, rhs))
    }

    fn __xor__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} ^ {:?}", lhs, rhs))
    }

    fn __or__(lhs: &PyObjectRef, rhs: &PyObjectRef) -> PyResult<String> {
        Ok(format!("{:?} | {:?}", lhs, rhs))
    }
}

#[test]
fn binary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|_| BinaryArithmetic {}).unwrap();
    py_run!(py, c, "assert c + c == 'BA + BA'");
    py_run!(py, c, "assert c + 1 == 'BA + 1'");
    py_run!(py, c, "assert 1 + c == '1 + BA'");
    py_run!(py, c, "assert c - 1 == 'BA - 1'");
    py_run!(py, c, "assert 1 - c == '1 - BA'");
    py_run!(py, c, "assert c * 1 == 'BA * 1'");
    py_run!(py, c, "assert 1 * c == '1 * BA'");

    py_run!(py, c, "assert c << 1 == 'BA << 1'");
    py_run!(py, c, "assert 1 << c == '1 << BA'");
    py_run!(py, c, "assert c >> 1 == 'BA >> 1'");
    py_run!(py, c, "assert 1 >> c == '1 >> BA'");
    py_run!(py, c, "assert c & 1 == 'BA & 1'");
    py_run!(py, c, "assert 1 & c == '1 & BA'");
    py_run!(py, c, "assert c ^ 1 == 'BA ^ 1'");
    py_run!(py, c, "assert 1 ^ c == '1 ^ BA'");
    py_run!(py, c, "assert c | 1 == 'BA | 1'");
    py_run!(py, c, "assert 1 | c == '1 | BA'");
}

#[pyclass]
struct RichComparisons {}

#[pyproto]
impl PyObjectProtocol for RichComparisons {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC")
    }

    fn __richcmp__(&self, other: &PyObjectRef, op: CompareOp) -> PyResult<String> {
        match op {
            CompareOp::Lt => Ok(format!("{} < {:?}", self.__repr__().unwrap(), other)),
            CompareOp::Le => Ok(format!("{} <= {:?}", self.__repr__().unwrap(), other)),
            CompareOp::Eq => Ok(format!("{} == {:?}", self.__repr__().unwrap(), other)),
            CompareOp::Ne => Ok(format!("{} != {:?}", self.__repr__().unwrap(), other)),
            CompareOp::Gt => Ok(format!("{} > {:?}", self.__repr__().unwrap(), other)),
            CompareOp::Ge => Ok(format!("{} >= {:?}", self.__repr__().unwrap(), other)),
        }
    }
}

#[pyclass]
struct RichComparisons2 {}

#[pyproto]
impl PyObjectProtocol for RichComparisons2 {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC2")
    }

    fn __richcmp__(&self, _other: &PyObjectRef, op: CompareOp) -> PyResult<PyObject> {
        let gil = GILGuard::acquire();
        match op {
            CompareOp::Eq => Ok(true.to_object(gil.python())),
            CompareOp::Ne => Ok(false.to_object(gil.python())),
            _ => Ok(gil.python().NotImplemented()),
        }
    }
}

#[test]
fn rich_comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|_| RichComparisons {}).unwrap();
    py_run!(py, c, "assert (c < c) == 'RC < RC'");
    py_run!(py, c, "assert (c < 1) == 'RC < 1'");
    py_run!(py, c, "assert (1 < c) == 'RC > 1'");
    py_run!(py, c, "assert (c <= c) == 'RC <= RC'");
    py_run!(py, c, "assert (c <= 1) == 'RC <= 1'");
    py_run!(py, c, "assert (1 <= c) == 'RC >= 1'");
    py_run!(py, c, "assert (c == c) == 'RC == RC'");
    py_run!(py, c, "assert (c == 1) == 'RC == 1'");
    py_run!(py, c, "assert (1 == c) == 'RC == 1'");
    py_run!(py, c, "assert (c != c) == 'RC != RC'");
    py_run!(py, c, "assert (c != 1) == 'RC != 1'");
    py_run!(py, c, "assert (1 != c) == 'RC != 1'");
    py_run!(py, c, "assert (c > c) == 'RC > RC'");
    py_run!(py, c, "assert (c > 1) == 'RC > 1'");
    py_run!(py, c, "assert (1 > c) == 'RC < 1'");
    py_run!(py, c, "assert (c >= c) == 'RC >= RC'");
    py_run!(py, c, "assert (c >= 1) == 'RC >= 1'");
    py_run!(py, c, "assert (1 >= c) == 'RC <= 1'");
}

#[test]
#[cfg(Py_3)]
fn rich_comparisons_python_3_type_error() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c2 = py.init(|_| RichComparisons2 {}).unwrap();
    py_expect_exception!(py, c2, "c2 < c2", TypeError);
    py_expect_exception!(py, c2, "c2 < 1", TypeError);
    py_expect_exception!(py, c2, "1 < c2", TypeError);
    py_expect_exception!(py, c2, "c2 <= c2", TypeError);
    py_expect_exception!(py, c2, "c2 <= 1", TypeError);
    py_expect_exception!(py, c2, "1 <= c2", TypeError);
    py_run!(py, c2, "assert (c2 == c2) == True");
    py_run!(py, c2, "assert (c2 == 1) == True");
    py_run!(py, c2, "assert (1 == c2) == True");
    py_run!(py, c2, "assert (c2 != c2) == False");
    py_run!(py, c2, "assert (c2 != 1) == False");
    py_run!(py, c2, "assert (1 != c2) == False");
    py_expect_exception!(py, c2, "c2 > c2", TypeError);
    py_expect_exception!(py, c2, "c2 > 1", TypeError);
    py_expect_exception!(py, c2, "1 > c2", TypeError);
    py_expect_exception!(py, c2, "c2 >= c2", TypeError);
    py_expect_exception!(py, c2, "c2 >= 1", TypeError);
    py_expect_exception!(py, c2, "1 >= c2", TypeError);
}
