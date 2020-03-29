#![feature(specialization)]

use pyo3::class::basic::CompareOp;
use pyo3::class::*;
use pyo3::prelude::*;
use pyo3::py_run;

mod common;

#[pyclass]
struct UnaryArithmetic {
    inner: f64,
}

impl UnaryArithmetic {
    fn new(value: f64) -> Self {
        UnaryArithmetic { inner: value }
    }
}

#[pyproto]
impl PyObjectProtocol for UnaryArithmetic {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("UA({})", self.inner))
    }
}

#[pyproto]
impl PyNumberProtocol for UnaryArithmetic {
    fn __neg__(&self) -> PyResult<Self> {
        Ok(Self::new(-self.inner))
    }

    fn __pos__(&self) -> PyResult<Self> {
        Ok(Self::new(self.inner))
    }

    fn __abs__(&self) -> PyResult<Self> {
        Ok(Self::new(self.inner.abs()))
    }

    fn __round__(&self, _ndigits: Option<u32>) -> PyResult<Self> {
        Ok(Self::new(self.inner.round()))
    }
}

#[test]
fn unary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, UnaryArithmetic::new(2.7)).unwrap();
    py_run!(py, c, "assert repr(-c) == 'UA(-2.7)'");
    py_run!(py, c, "assert repr(+c) == 'UA(2.7)'");
    py_run!(py, c, "assert repr(abs(c)) == 'UA(2.7)'");
    py_run!(py, c, "assert repr(round(c)) == 'UA(3)'");
    py_run!(py, c, "assert repr(round(c, 1)) == 'UA(3)'");
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

    fn __ipow__(&mut self, other: u32) -> PyResult<()> {
        self.value = self.value.pow(other);
        Ok(())
    }
}

#[test]
fn inplace_operations() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let init = |value, code| {
        let c = PyCell::new(py, InPlaceOperations { value }).unwrap();
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
    init(3, "d = c; c **= 4; assert repr(c) == repr(d) == 'IPO(81)'");
    init(
        3,
        "d = c; c.__ipow__(4); assert repr(c) == repr(d) == 'IPO(81)'",
    );
}

#[pyproto]
impl PyNumberProtocol for BinaryArithmetic {
    fn __add__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", lhs, rhs))
    }

    fn __sub__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", lhs, rhs))
    }

    fn __mul__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} * {:?}", lhs, rhs))
    }

    fn __lshift__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} << {:?}", lhs, rhs))
    }

    fn __rshift__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} >> {:?}", lhs, rhs))
    }

    fn __and__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} & {:?}", lhs, rhs))
    }

    fn __xor__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} ^ {:?}", lhs, rhs))
    }

    fn __or__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} | {:?}", lhs, rhs))
    }

    fn __pow__(lhs: &PyAny, rhs: &PyAny, mod_: Option<u32>) -> PyResult<String> {
        Ok(format!("{:?} ** {:?} (mod: {:?})", lhs, rhs, mod_))
    }
}

#[test]
fn binary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, BinaryArithmetic {}).unwrap();
    py_run!(py, c, "assert c + c == 'BA + BA'");
    py_run!(py, c, "assert c.__add__(c) == 'BA + BA'");
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
    py_run!(py, c, "assert c ** 1 == 'BA ** 1 (mod: None)'");
    py_run!(py, c, "assert 1 ** c == '1 ** BA (mod: None)'");

    py_run!(py, c, "assert pow(c, 1, 100) == 'BA ** 1 (mod: Some(100))'");
}

#[pyclass]
struct RhsArithmetic {}

#[pyproto]
impl PyNumberProtocol for RhsArithmetic {
    fn __radd__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} + RA", other))
    }

    fn __rsub__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} - RA", other))
    }

    fn __rmul__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} * RA", other))
    }

    fn __rlshift__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} << RA", other))
    }

    fn __rrshift__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} >> RA", other))
    }

    fn __rand__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} & RA", other))
    }

    fn __rxor__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} ^ RA", other))
    }

    fn __ror__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} | RA", other))
    }

    fn __rpow__(&self, other: &PyAny, _mod: Option<&'p PyAny>) -> PyResult<String> {
        Ok(format!("{:?} ** RA", other))
    }
}

#[test]
fn rhs_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, RhsArithmetic {}).unwrap();
    py_run!(py, c, "assert c.__radd__(1) == '1 + RA'");
    py_run!(py, c, "assert 1 + c == '1 + RA'");
    py_run!(py, c, "assert c.__rsub__(1) == '1 - RA'");
    py_run!(py, c, "assert 1 - c == '1 - RA'");
    py_run!(py, c, "assert c.__rmul__(1) == '1 * RA'");
    py_run!(py, c, "assert 1 * c == '1 * RA'");
    py_run!(py, c, "assert c.__rlshift__(1) == '1 << RA'");
    py_run!(py, c, "assert 1 << c == '1 << RA'");
    py_run!(py, c, "assert c.__rrshift__(1) == '1 >> RA'");
    py_run!(py, c, "assert 1 >> c == '1 >> RA'");
    py_run!(py, c, "assert c.__rand__(1) == '1 & RA'");
    py_run!(py, c, "assert 1 & c == '1 & RA'");
    py_run!(py, c, "assert c.__rxor__(1) == '1 ^ RA'");
    py_run!(py, c, "assert 1 ^ c == '1 ^ RA'");
    py_run!(py, c, "assert c.__ror__(1) == '1 | RA'");
    py_run!(py, c, "assert 1 | c == '1 | RA'");
    py_run!(py, c, "assert c.__rpow__(1) == '1 ** RA'");
    py_run!(py, c, "assert 1 ** c == '1 ** RA'");
}

#[pyclass]
struct LhsAndRhsArithmetic {}

#[pyproto]
impl PyNumberProtocol for LhsAndRhsArithmetic {
    fn __radd__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} + RA", other))
    }

    fn __rsub__(&self, other: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} - RA", other))
    }

    fn __rpow__(&self, other: &PyAny, _mod: Option<&'p PyAny>) -> PyResult<String> {
        Ok(format!("{:?} ** RA", other))
    }

    fn __add__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", lhs, rhs))
    }

    fn __sub__(lhs: &PyAny, rhs: &PyAny) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", lhs, rhs))
    }

    fn __pow__(lhs: &PyAny, rhs: &PyAny, _mod: Option<u32>) -> PyResult<String> {
        Ok(format!("{:?} ** {:?}", lhs, rhs))
    }
}

#[pyproto]
impl PyObjectProtocol for LhsAndRhsArithmetic {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[test]
fn lhs_override_rhs() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = PyCell::new(py, LhsAndRhsArithmetic {}).unwrap();
    // Not overrided
    py_run!(py, c, "assert c.__radd__(1) == '1 + RA'");
    py_run!(py, c, "assert c.__rsub__(1) == '1 - RA'");
    py_run!(py, c, "assert c.__rpow__(1) == '1 ** RA'");
    // Overrided
    py_run!(py, c, "assert 1 + c == '1 + BA'");
    py_run!(py, c, "assert 1 - c == '1 - BA'");
    py_run!(py, c, "assert 1 ** c == '1 ** BA'");
}

#[pyclass]
struct RichComparisons {}

#[pyproto]
impl PyObjectProtocol for RichComparisons {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC")
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<String> {
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

    fn __richcmp__(&self, _other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
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

    let c = PyCell::new(py, RichComparisons {}).unwrap();
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
fn rich_comparisons_python_3_type_error() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c2 = PyCell::new(py, RichComparisons2 {}).unwrap();
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
