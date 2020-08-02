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
    fn __repr__(&self) -> String {
        format!("UA({})", self.inner)
    }
}

#[pyproto]
impl PyNumberProtocol for UnaryArithmetic {
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
    fn __repr__(&self) -> &'static str {
        "BA"
    }
}

#[pyclass]
struct InPlaceOperations {
    value: u32,
}

#[pyproto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self) -> String {
        format!("IPO({:?})", self.value)
    }
}

#[pyproto]
impl PyNumberProtocol for InPlaceOperations {
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

    fn __ipow__(&mut self, other: u32) {
        self.value = self.value.pow(other);
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
    fn __add__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} + {:?}", lhs, rhs)
    }

    fn __sub__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} - {:?}", lhs, rhs)
    }

    fn __mul__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} * {:?}", lhs, rhs)
    }

    fn __lshift__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} << {:?}", lhs, rhs)
    }

    fn __rshift__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} >> {:?}", lhs, rhs)
    }

    fn __and__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} & {:?}", lhs, rhs)
    }

    fn __xor__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} ^ {:?}", lhs, rhs)
    }

    fn __or__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} | {:?}", lhs, rhs)
    }

    fn __pow__(lhs: &PyAny, rhs: &PyAny, mod_: Option<u32>) -> String {
        format!("{:?} ** {:?} (mod: {:?})", lhs, rhs, mod_)
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

    fn __rpow__(&self, other: &PyAny, _mod: Option<&'p PyAny>) -> String {
        format!("{:?} ** RA", other)
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
    fn __radd__(&self, other: &PyAny) -> String {
        format!("{:?} + RA", other)
    }

    fn __rsub__(&self, other: &PyAny) -> String {
        format!("{:?} - RA", other)
    }

    fn __rpow__(&self, other: &PyAny, _mod: Option<&'p PyAny>) -> String {
        format!("{:?} ** RA", other)
    }

    fn __add__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} + {:?}", lhs, rhs)
    }

    fn __sub__(lhs: &PyAny, rhs: &PyAny) -> String {
        format!("{:?} - {:?}", lhs, rhs)
    }

    fn __pow__(lhs: &PyAny, rhs: &PyAny, _mod: Option<u32>) -> String {
        format!("{:?} ** {:?}", lhs, rhs)
    }
}

#[pyproto]
impl PyObjectProtocol for LhsAndRhsArithmetic {
    fn __repr__(&self) -> &'static str {
        "BA"
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

#[pyproto]
impl PyObjectProtocol for RichComparisons2 {
    fn __repr__(&self) -> &'static str {
        "RC2"
    }

    fn __richcmp__(&self, _other: &PyAny, op: CompareOp) -> PyObject {
        let gil = GILGuard::acquire();
        let py = gil.python();
        match op {
            CompareOp::Eq => true.into_py(py),
            CompareOp::Ne => false.into_py(py),
            _ => py.NotImplemented(),
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
}

// Checks that binary operations for which the arguments don't match the
// required type, return NotImplemented.
mod return_not_implemented {
    use super::*;

    #[pyclass]
    struct RichComparisonToSelf {}

    #[pyproto]
    impl<'p> PyObjectProtocol<'p> for RichComparisonToSelf {
        fn __repr__(&self) -> &'static str {
            "RC_Self"
        }

        fn __richcmp__(&self, other: PyRef<'p, Self>, _op: CompareOp) -> PyObject {
            other.py().None()
        }
    }

    #[pyproto]
    impl<'p> PyNumberProtocol<'p> for RichComparisonToSelf {
        fn __add__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __sub__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __mul__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __matmul__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __truediv__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __floordiv__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __mod__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __pow__(lhs: &'p PyAny, _other: u8, _modulo: Option<u8>) -> &'p PyAny {
            lhs
        }
        fn __lshift__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __rshift__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
        fn __divmod__(lhs: &'p PyAny, _other: PyRef<'p, Self>) -> &'p PyAny {
            lhs
        }
    }

    fn _test_bool_operator(operator: &str) {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let c2 = PyCell::new(py, RichComparisonToSelf {}).unwrap();
        py_run!(
            py,
            c2,
            &format!(
                "\
class Other:
    def __eq__(self, other):
        return True
    __ne__ = __lt__ = __le__ = __gt__ = __ge__ = __eq__

assert (c2 {} Other()) is True",
                operator
            )
        );
    }

    fn _test_logical_operator(operator: &str) {
        _test_bool_operator(operator);

        let gil = Python::acquire_gil();
        let py = gil.python();
        let c2 = PyCell::new(py, RichComparisonToSelf {}).unwrap();
        py_expect_exception!(
            py,
            c2,
            &format!("class Other: pass\nc2 {} Other()", operator),
            PyTypeError
        )
    }

    fn _test_binary_num_operator(operator: &str) {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let c2 = PyCell::new(py, RichComparisonToSelf {}).unwrap();
        py_run!(
            py,
            c2,
            &format!(
                "\
class Other:
    def __radd__(self, other):
        return other
    __rand__ = __ror__ = __rxor__ = __radd__
    __rsub__ = __rmul__ = __rtruediv__ = __rfloordiv__ = __rpow__ = __radd__
    __rmatmul__ = __rlshift__ = __rrshift__ = __rmod__ = __rdivmod__ = __radd__

assert (c2 {} Other()) is c2",
                operator
            )
        );

        py_expect_exception!(
            py,
            c2,
            &format!("class Other: pass\nc2 {} Other()", operator),
            PyTypeError
        )
    }

    #[test]
    fn equality() {
        _test_bool_operator("==");
        _test_bool_operator("!=");
    }

    #[test]
    fn ordering() {
        _test_logical_operator("<");
        _test_logical_operator("<=");
        _test_logical_operator(">");
        _test_logical_operator(">=");
    }

    #[test]
    fn bitwise() {
        _test_binary_num_operator("&");
        _test_binary_num_operator("|");
        _test_binary_num_operator("^");
        _test_binary_num_operator("<<");
        _test_binary_num_operator(">>");
    }

    #[test]
    fn arith() {
        _test_binary_num_operator("+");
        _test_binary_num_operator("-");
        _test_binary_num_operator("*");
        _test_binary_num_operator("@");
        _test_binary_num_operator("/");
        _test_binary_num_operator("//");
        _test_binary_num_operator("%");
        _test_binary_num_operator("**");
    }
}
