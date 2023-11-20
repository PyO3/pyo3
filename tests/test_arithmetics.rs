#![cfg(feature = "macros")]

use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::py_run;

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct UnaryArithmetic {
    inner: f64,
}

#[pymethods]
impl UnaryArithmetic {
    #[new]
    fn new(value: f64) -> Self {
        UnaryArithmetic { inner: value }
    }

    fn __repr__(&self) -> String {
        format!("UA({})", self.inner)
    }

    fn __neg__(&self) -> Self {
        Self::new(-self.inner)
    }

    fn __pos__(&self) -> Self {
        Self::new(self.inner)
    }

    fn __abs__(&self) -> Self {
        Self::new(self.inner.abs())
    }

    fn __round__(&self, _ndigits: Option<u32>) -> Self {
        Self::new(self.inner.round())
    }
}

#[test]
fn unary_arithmetic() {
    Python::with_gil(|py| {
        let c = PyCell::new(py, UnaryArithmetic::new(2.7)).unwrap();
        py_run!(py, c, "assert repr(-c) == 'UA(-2.7)'");
        py_run!(py, c, "assert repr(+c) == 'UA(2.7)'");
        py_run!(py, c, "assert repr(abs(c)) == 'UA(2.7)'");
        py_run!(py, c, "assert repr(round(c)) == 'UA(3)'");
        py_run!(py, c, "assert repr(round(c, 1)) == 'UA(3)'");
    });
}

#[pyclass]
struct Indexable(i32);

#[pymethods]
impl Indexable {
    fn __index__(&self) -> i32 {
        self.0
    }

    fn __int__(&self) -> i32 {
        self.0
    }

    fn __float__(&self) -> f64 {
        f64::from(self.0)
    }

    fn __invert__(&self) -> Self {
        Self(!self.0)
    }
}

#[test]
fn indexable() {
    Python::with_gil(|py| {
        let i = PyCell::new(py, Indexable(5)).unwrap();
        py_run!(py, i, "assert int(i) == 5");
        py_run!(py, i, "assert [0, 1, 2, 3, 4, 5][i] == 5");
        py_run!(py, i, "assert float(i) == 5.0");
        py_run!(py, i, "assert int(~i) == -6");
    })
}

#[pyclass]
struct InPlaceOperations {
    value: u32,
}

#[pymethods]
impl InPlaceOperations {
    fn __repr__(&self) -> String {
        format!("IPO({:?})", self.value)
    }

    fn __iadd__(&mut self, other: u32) {
        self.value += other;
    }

    fn __isub__(&mut self, other: u32) {
        self.value -= other;
    }

    fn __imul__(&mut self, other: u32) {
        self.value *= other;
    }

    fn __ilshift__(&mut self, other: u32) {
        self.value <<= other;
    }

    fn __irshift__(&mut self, other: u32) {
        self.value >>= other;
    }

    fn __iand__(&mut self, other: u32) {
        self.value &= other;
    }

    fn __ixor__(&mut self, other: u32) {
        self.value ^= other;
    }

    fn __ior__(&mut self, other: u32) {
        self.value |= other;
    }

    fn __ipow__(&mut self, other: u32, _modulo: Option<u32>) {
        self.value = self.value.pow(other);
    }
}

#[test]
fn inplace_operations() {
    Python::with_gil(|py| {
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
    });
}

#[pyclass]
struct BinaryArithmetic {}

#[pymethods]
impl BinaryArithmetic {
    fn __repr__(&self) -> &'static str {
        "BA"
    }

    fn __add__(&self, rhs: &PyAny) -> String {
        format!("BA + {:?}", rhs)
    }

    fn __sub__(&self, rhs: &PyAny) -> String {
        format!("BA - {:?}", rhs)
    }

    fn __mul__(&self, rhs: &PyAny) -> String {
        format!("BA * {:?}", rhs)
    }

    fn __lshift__(&self, rhs: &PyAny) -> String {
        format!("BA << {:?}", rhs)
    }

    fn __rshift__(&self, rhs: &PyAny) -> String {
        format!("BA >> {:?}", rhs)
    }

    fn __and__(&self, rhs: &PyAny) -> String {
        format!("BA & {:?}", rhs)
    }

    fn __xor__(&self, rhs: &PyAny) -> String {
        format!("BA ^ {:?}", rhs)
    }

    fn __or__(&self, rhs: &PyAny) -> String {
        format!("BA | {:?}", rhs)
    }

    fn __pow__(&self, rhs: &PyAny, mod_: Option<u32>) -> String {
        format!("BA ** {:?} (mod: {:?})", rhs, mod_)
    }
}

#[test]
fn binary_arithmetic() {
    Python::with_gil(|py| {
        let c = PyCell::new(py, BinaryArithmetic {}).unwrap();
        py_run!(py, c, "assert c + c == 'BA + BA'");
        py_run!(py, c, "assert c.__add__(c) == 'BA + BA'");
        py_run!(py, c, "assert c + 1 == 'BA + 1'");
        py_run!(py, c, "assert c - 1 == 'BA - 1'");
        py_run!(py, c, "assert c * 1 == 'BA * 1'");
        py_run!(py, c, "assert c << 1 == 'BA << 1'");
        py_run!(py, c, "assert c >> 1 == 'BA >> 1'");
        py_run!(py, c, "assert c & 1 == 'BA & 1'");
        py_run!(py, c, "assert c ^ 1 == 'BA ^ 1'");
        py_run!(py, c, "assert c | 1 == 'BA | 1'");
        py_run!(py, c, "assert c ** 1 == 'BA ** 1 (mod: None)'");

        // Class with __add__ only should not allow the reverse op;
        // this is consistent with Python classes.

        py_expect_exception!(py, c, "1 + c", PyTypeError);
        py_expect_exception!(py, c, "1 - c", PyTypeError);
        py_expect_exception!(py, c, "1 * c", PyTypeError);
        py_expect_exception!(py, c, "1 << c", PyTypeError);
        py_expect_exception!(py, c, "1 >> c", PyTypeError);
        py_expect_exception!(py, c, "1 & c", PyTypeError);
        py_expect_exception!(py, c, "1 ^ c", PyTypeError);
        py_expect_exception!(py, c, "1 | c", PyTypeError);
        py_expect_exception!(py, c, "1 ** c", PyTypeError);

        py_run!(py, c, "assert pow(c, 1, 100) == 'BA ** 1 (mod: Some(100))'");
    });
}

#[pyclass]
struct RhsArithmetic {}

#[pymethods]
impl RhsArithmetic {
    fn __radd__(&self, other: &PyAny) -> String {
        format!("{:?} + RA", other)
    }

    fn __rsub__(&self, other: &PyAny) -> String {
        format!("{:?} - RA", other)
    }

    fn __rmul__(&self, other: &PyAny) -> String {
        format!("{:?} * RA", other)
    }

    fn __rlshift__(&self, other: &PyAny) -> String {
        format!("{:?} << RA", other)
    }

    fn __rrshift__(&self, other: &PyAny) -> String {
        format!("{:?} >> RA", other)
    }

    fn __rand__(&self, other: &PyAny) -> String {
        format!("{:?} & RA", other)
    }

    fn __rxor__(&self, other: &PyAny) -> String {
        format!("{:?} ^ RA", other)
    }

    fn __ror__(&self, other: &PyAny) -> String {
        format!("{:?} | RA", other)
    }

    fn __rpow__(&self, other: &PyAny, _mod: Option<&PyAny>) -> String {
        format!("{:?} ** RA", other)
    }
}

#[test]
fn rhs_arithmetic() {
    Python::with_gil(|py| {
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
    });
}

#[pyclass]
struct LhsAndRhs {}

impl std::fmt::Debug for LhsAndRhs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LR")
    }
}

#[pymethods]
impl LhsAndRhs {
    // fn __repr__(&self) -> &'static str {
    //     "BA"
    // }

    fn __add__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} + {:?}", lhs, rhs)
    }

    fn __sub__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} - {:?}", lhs, rhs)
    }

    fn __mul__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} * {:?}", lhs, rhs)
    }

    fn __lshift__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} << {:?}", lhs, rhs)
    }

    fn __rshift__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} >> {:?}", lhs, rhs)
    }

    fn __and__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} & {:?}", lhs, rhs)
    }

    fn __xor__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} ^ {:?}", lhs, rhs)
    }

    fn __or__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} | {:?}", lhs, rhs)
    }

    fn __pow__(lhs: PyRef<'_, Self>, rhs: &PyAny, _mod: Option<usize>) -> String {
        format!("{:?} ** {:?}", lhs, rhs)
    }

    fn __matmul__(lhs: PyRef<'_, Self>, rhs: &PyAny) -> String {
        format!("{:?} @ {:?}", lhs, rhs)
    }

    fn __radd__(&self, other: &PyAny) -> String {
        format!("{:?} + RA", other)
    }

    fn __rsub__(&self, other: &PyAny) -> String {
        format!("{:?} - RA", other)
    }

    fn __rmul__(&self, other: &PyAny) -> String {
        format!("{:?} * RA", other)
    }

    fn __rlshift__(&self, other: &PyAny) -> String {
        format!("{:?} << RA", other)
    }

    fn __rrshift__(&self, other: &PyAny) -> String {
        format!("{:?} >> RA", other)
    }

    fn __rand__(&self, other: &PyAny) -> String {
        format!("{:?} & RA", other)
    }

    fn __rxor__(&self, other: &PyAny) -> String {
        format!("{:?} ^ RA", other)
    }

    fn __ror__(&self, other: &PyAny) -> String {
        format!("{:?} | RA", other)
    }

    fn __rpow__(&self, other: &PyAny, _mod: Option<&PyAny>) -> String {
        format!("{:?} ** RA", other)
    }

    fn __rmatmul__(&self, other: &PyAny) -> String {
        format!("{:?} @ RA", other)
    }

    fn __rtruediv__(&self, other: &PyAny) -> String {
        format!("{:?} / RA", other)
    }

    fn __rfloordiv__(&self, other: &PyAny) -> String {
        format!("{:?} // RA", other)
    }
}

#[test]
fn lhs_fellback_to_rhs() {
    Python::with_gil(|py| {
        let c = PyCell::new(py, LhsAndRhs {}).unwrap();
        // If the light hand value is `LhsAndRhs`, LHS is used.
        py_run!(py, c, "assert c + 1 == 'LR + 1'");
        py_run!(py, c, "assert c - 1 == 'LR - 1'");
        py_run!(py, c, "assert c * 1 == 'LR * 1'");
        py_run!(py, c, "assert c << 1 == 'LR << 1'");
        py_run!(py, c, "assert c >> 1 == 'LR >> 1'");
        py_run!(py, c, "assert c & 1 == 'LR & 1'");
        py_run!(py, c, "assert c ^ 1 == 'LR ^ 1'");
        py_run!(py, c, "assert c | 1 == 'LR | 1'");
        py_run!(py, c, "assert c ** 1 == 'LR ** 1'");
        py_run!(py, c, "assert c @ 1 == 'LR @ 1'");
        // Fellback to RHS because of type mismatching
        py_run!(py, c, "assert 1 + c == '1 + RA'");
        py_run!(py, c, "assert 1 - c == '1 - RA'");
        py_run!(py, c, "assert 1 * c == '1 * RA'");
        py_run!(py, c, "assert 1 << c == '1 << RA'");
        py_run!(py, c, "assert 1 >> c == '1 >> RA'");
        py_run!(py, c, "assert 1 & c == '1 & RA'");
        py_run!(py, c, "assert 1 ^ c == '1 ^ RA'");
        py_run!(py, c, "assert 1 | c == '1 | RA'");
        py_run!(py, c, "assert 1 ** c == '1 ** RA'");
        py_run!(py, c, "assert 1 @ c == '1 @ RA'");
    });
}

#[pyclass]
struct RichComparisons {}

#[pymethods]
impl RichComparisons {
    fn __repr__(&self) -> &'static str {
        "RC"
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> String {
        match op {
            CompareOp::Lt => format!("{} < {:?}", self.__repr__(), other),
            CompareOp::Le => format!("{} <= {:?}", self.__repr__(), other),
            CompareOp::Eq => format!("{} == {:?}", self.__repr__(), other),
            CompareOp::Ne => format!("{} != {:?}", self.__repr__(), other),
            CompareOp::Gt => format!("{} > {:?}", self.__repr__(), other),
            CompareOp::Ge => format!("{} >= {:?}", self.__repr__(), other),
        }
    }
}

#[pyclass]
struct RichComparisons2 {}

#[pymethods]
impl RichComparisons2 {
    fn __repr__(&self) -> &'static str {
        "RC2"
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyObject {
        match op {
            CompareOp::Eq => true.into_py(other.py()),
            CompareOp::Ne => false.into_py(other.py()),
            _ => other.py().NotImplemented().into(),
        }
    }
}

#[test]
fn rich_comparisons() {
    Python::with_gil(|py| {
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
    });
}

#[test]
fn rich_comparisons_python_3_type_error() {
    Python::with_gil(|py| {
        let c2 = PyCell::new(py, RichComparisons2 {}).unwrap();
        py_expect_exception!(py, c2, "c2 < c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 < 1", PyTypeError);
        py_expect_exception!(py, c2, "1 < c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 <= c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 <= 1", PyTypeError);
        py_expect_exception!(py, c2, "1 <= c2", PyTypeError);
        py_run!(py, c2, "assert (c2 == c2) == True");
        py_run!(py, c2, "assert (c2 == 1) == True");
        py_run!(py, c2, "assert (1 == c2) == True");
        py_run!(py, c2, "assert (c2 != c2) == False");
        py_run!(py, c2, "assert (c2 != 1) == False");
        py_run!(py, c2, "assert (1 != c2) == False");
        py_expect_exception!(py, c2, "c2 > c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 > 1", PyTypeError);
        py_expect_exception!(py, c2, "1 > c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 >= c2", PyTypeError);
        py_expect_exception!(py, c2, "c2 >= 1", PyTypeError);
        py_expect_exception!(py, c2, "1 >= c2", PyTypeError);
    });
}

// Checks that binary operations for which the arguments don't match the
// required type, return NotImplemented.
mod return_not_implemented {
    use super::*;

    #[pyclass]
    struct RichComparisonToSelf {}

    #[pymethods]
    impl RichComparisonToSelf {
        fn __repr__(&self) -> &'static str {
            "RC_Self"
        }

        fn __richcmp__(&self, other: PyRef<'_, Self>, _op: CompareOp) -> PyObject {
            other.py().None().into()
        }

        fn __add__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __sub__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __mul__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __matmul__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __truediv__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __floordiv__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __mod__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __pow__(slf: PyRef<'_, Self>, _other: u8, _modulo: Option<u8>) -> PyRef<'_, Self> {
            slf
        }
        fn __lshift__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __rshift__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __divmod__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __and__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __or__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }
        fn __xor__<'p>(slf: PyRef<'p, Self>, _other: PyRef<'p, Self>) -> PyRef<'p, Self> {
            slf
        }

        // Inplace assignments
        fn __iadd__(&mut self, _other: PyRef<'_, Self>) {}
        fn __isub__(&mut self, _other: PyRef<'_, Self>) {}
        fn __imul__(&mut self, _other: PyRef<'_, Self>) {}
        fn __imatmul__(&mut self, _other: PyRef<'_, Self>) {}
        fn __itruediv__(&mut self, _other: PyRef<'_, Self>) {}
        fn __ifloordiv__(&mut self, _other: PyRef<'_, Self>) {}
        fn __imod__(&mut self, _other: PyRef<'_, Self>) {}
        fn __ilshift__(&mut self, _other: PyRef<'_, Self>) {}
        fn __irshift__(&mut self, _other: PyRef<'_, Self>) {}
        fn __iand__(&mut self, _other: PyRef<'_, Self>) {}
        fn __ior__(&mut self, _other: PyRef<'_, Self>) {}
        fn __ixor__(&mut self, _other: PyRef<'_, Self>) {}
        fn __ipow__(&mut self, _other: PyRef<'_, Self>, _modulo: Option<u8>) {}
    }

    fn _test_binary_dunder(dunder: &str) {
        Python::with_gil(|py| {
            let c2 = PyCell::new(py, RichComparisonToSelf {}).unwrap();
            py_run!(
                py,
                c2,
                &format!(
                    "class Other: pass\nassert c2.__{}__(Other()) is NotImplemented",
                    dunder
                )
            );
        });
    }

    fn _test_binary_operator(operator: &str, dunder: &str) {
        _test_binary_dunder(dunder);

        Python::with_gil(|py| {
            let c2 = PyCell::new(py, RichComparisonToSelf {}).unwrap();
            py_expect_exception!(
                py,
                c2,
                &format!("class Other: pass\nc2 {} Other()", operator),
                PyTypeError
            );
        });
    }

    fn _test_inplace_binary_operator(operator: &str, dunder: &str) {
        _test_binary_operator(operator, dunder);
    }

    #[test]
    fn equality() {
        _test_binary_dunder("eq");
        _test_binary_dunder("ne");
    }

    #[test]
    fn ordering() {
        _test_binary_operator("<", "lt");
        _test_binary_operator("<=", "le");
        _test_binary_operator(">", "gt");
        _test_binary_operator(">=", "ge");
    }

    #[test]
    fn bitwise() {
        _test_binary_operator("&", "and");
        _test_binary_operator("|", "or");
        _test_binary_operator("^", "xor");
        _test_binary_operator("<<", "lshift");
        _test_binary_operator(">>", "rshift");
    }

    #[test]
    fn arith() {
        _test_binary_operator("+", "add");
        _test_binary_operator("-", "sub");
        _test_binary_operator("*", "mul");
        _test_binary_operator("@", "matmul");
        _test_binary_operator("/", "truediv");
        _test_binary_operator("//", "floordiv");
        _test_binary_operator("%", "mod");
        _test_binary_operator("**", "pow");
    }

    #[test]
    fn reverse_arith() {
        _test_binary_dunder("radd");
        _test_binary_dunder("rsub");
        _test_binary_dunder("rmul");
        _test_binary_dunder("rmatmul");
        _test_binary_dunder("rtruediv");
        _test_binary_dunder("rfloordiv");
        _test_binary_dunder("rmod");
        _test_binary_dunder("rdivmod");
        _test_binary_dunder("rpow");
    }

    #[test]
    fn inplace_bitwise() {
        _test_inplace_binary_operator("&=", "iand");
        _test_inplace_binary_operator("|=", "ior");
        _test_inplace_binary_operator("^=", "ixor");
        _test_inplace_binary_operator("<<=", "ilshift");
        _test_inplace_binary_operator(">>=", "irshift");
    }

    #[test]
    fn inplace_arith() {
        _test_inplace_binary_operator("+=", "iadd");
        _test_inplace_binary_operator("-=", "isub");
        _test_inplace_binary_operator("*=", "imul");
        _test_inplace_binary_operator("@=", "imatmul");
        _test_inplace_binary_operator("/=", "itruediv");
        _test_inplace_binary_operator("//=", "ifloordiv");
        _test_inplace_binary_operator("%=", "imod");
        _test_inplace_binary_operator("**=", "ipow");
    }
}
