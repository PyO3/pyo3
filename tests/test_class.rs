#![feature(proc_macro, specialization)]
#![allow(dead_code, unused_variables)]

extern crate pyo3;

use pyo3::*;
use std::{mem, isize, iter};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use pyo3::ffi;

macro_rules! py_run {
    ($py:expr, $val:ident, $code:expr) => {{
        let d = PyDict::new($py);
        d.set_item($py, stringify!($val), &$val).unwrap();
        //$py.run($code, None, Some(&d)).map_err(|e| e.print($py)).expect($code);
        $py.run($code, None, Some(&d)).expect($code);
    }}
}

macro_rules! py_assert {
    ($py:expr, $val:ident, $assertion:expr) => { py_run!($py, $val, concat!("assert ", $assertion)) };
}

macro_rules! py_expect_exception {
    ($py:expr, $val:ident, $code:expr, $err:ident) => {{
        let d = PyDict::new($py);
        d.set_item($py, stringify!($val), &$val).unwrap();
        let res = $py.run($code, None, Some(&d));
        let err = res.unwrap_err();
        if !err.matches($py, $py.get_type::<exc::$err>()) {
            panic!(format!("Expected {} but got {:?}", stringify!($err), err))
        }
    }}
}


#[py::class]
struct EmptyClass { }

#[test]
fn empty_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass>();
    // By default, don't allow creating instances from python.
    assert!(typeobj.call(py, NoArgs, None).is_err());

    py_assert!(py, typeobj, "typeobj.__name__ == 'EmptyClass'");
}

#[py::class]
struct EmptyClassInModule { }

#[test]
fn empty_class_in_module() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let module = PyModule::new(py, "test_module.nested").unwrap();
    module.add_class::<EmptyClassInModule>(py).unwrap();

    let ty = module.get(py, "EmptyClassInModule").unwrap();
    assert_eq!(ty.getattr(py, "__name__").unwrap().extract::<String>(py).unwrap(), "EmptyClassInModule");
    assert_eq!(ty.getattr(py, "__module__").unwrap().extract::<String>(py).unwrap(), "test_module.nested");
}

#[py::class]
struct EmptyClassWithNew { }

#[py::methods]
impl EmptyClassWithNew {
    #[__new__]
    fn __new__(_cls: &PyType, py: Python) -> PyResult<EmptyClassWithNew> {
        EmptyClassWithNew::create_instance(py)
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj.call(py, NoArgs, None).unwrap().cast_into::<EmptyClassWithNew>(py).is_ok());
}

#[py::class]
struct NewWithOneArg {
    _data: i32,
}
#[py::methods]
impl NewWithOneArg {
    #[new]
    fn __new__(_cls: &PyType, py: Python, arg: i32) -> PyResult<NewWithOneArg> {
        NewWithOneArg::create_instance(py, arg)
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let obj = typeobj.call(py, (42,), None).unwrap().cast_into::<NewWithOneArg>(py).unwrap();
    assert_eq!(*obj._data(py), 42);
}

#[py::class]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,
}

#[py::methods]
impl NewWithTwoArgs {
    #[new]
    fn __new__(_cls: &PyType, py: Python, arg1: i32, arg2: i32) -> PyResult<NewWithTwoArgs> {
        NewWithTwoArgs::create_instance(py, arg1, arg2)
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let obj = typeobj.call(py, (10, 20), None).unwrap().cast_into::<NewWithTwoArgs>(py).unwrap();
    assert_eq!(*obj._data1(py), 10);
    assert_eq!(*obj._data2(py), 20);
}

struct TestDropCall {
    drop_called: Arc<AtomicBool>
}
impl Drop for TestDropCall {
    fn drop(&mut self) {
        self.drop_called.store(true, Ordering::Relaxed);
    }
}

#[py::class]
struct DataIsDropped {
    member1: TestDropCall,
    member2: TestDropCall,
}

#[test]
fn data_is_dropped() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));
    let inst = DataIsDropped::create_instance(py,
        TestDropCall { drop_called: drop_called1.clone() },
        TestDropCall { drop_called: drop_called2.clone() });
    assert!(drop_called1.load(Ordering::Relaxed) == false);
    assert!(drop_called2.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called1.load(Ordering::Relaxed) == true);
    assert!(drop_called2.load(Ordering::Relaxed) == true);
}

#[py::class]
struct InstanceMethod {
    member: i32,
}

#[py::methods]
impl InstanceMethod {
    fn method(&self, py: Python) -> PyResult<i32> {
        Ok(*self.member(py))
    }
}

#[test]
fn instance_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = InstanceMethod::create_instance(py, 42).unwrap();
    assert!(obj.method(py).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method() == 42", None, Some(&d)).unwrap();
}

#[py::class]
struct InstanceMethodWithArgs {
    member: i32
}
#[py::methods]
impl InstanceMethodWithArgs {
    fn method(&self, py: Python, multiplier: i32) -> PyResult<i32> {
        Ok(*self.member(py) * multiplier)
    }
}

#[test]
fn instance_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = InstanceMethodWithArgs::create_instance(py, 7).unwrap();
    assert!(obj.method(py, 6).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method(3) == 21", None, Some(&d)).unwrap();
    py.run("assert obj.method(multiplier=6) == 42", None, Some(&d)).unwrap();
}

#[py::class]
struct ClassMethod {}
#[py::methods]
impl ClassMethod {
    #[new]
    fn __new__(cls: &PyType, py: Python) -> PyResult<ClassMethod> {
        ClassMethod::create_instance(py)
    }

    //#[classmethod]
    //def method(cls) -> PyResult<String> {
    //    Ok(format!("{}.method()!", cls.name(py)))
    //}
}

//#[test]
fn class_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<ClassMethod>()).unwrap();
    py.run("assert C.method() == 'ClassMethod.method()!'", None, Some(&d)).unwrap();
    py.run("assert C().method() == 'ClassMethod.method()!'", None, Some(&d)).unwrap();
}

//py_class!(class ClassMethodWithArgs |py| {
//    @classmethod
//    def method(cls, input: &str) -> PyResult<String> {
//        Ok(format!("{}.method({})", cls.name(py), input))
//    }
//});

//#[test]
/*fn class_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<ClassMethodWithArgs>()).unwrap();
    py.run("assert C.method('abc') == 'ClassMethodWithArgs.method(abc)'", None, Some(&d)).unwrap();
}*/

#[py::class]
struct StaticMethod {}

#[py::methods]
impl StaticMethod {
    #[new]
    fn __new__(cls: &PyType, py: Python) -> PyResult<StaticMethod> {
        StaticMethod::create_instance(py)
    }

    //#[staticmethod]
    //fn method(py: Python) -> PyResult<&'static str> {
    //    Ok("StaticMethod.method()!")
    //}
}

//#[test]
/*fn static_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    assert_eq!(StaticMethod::method(py).unwrap(), "StaticMethod.method()!");
    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<StaticMethod>()).unwrap();
    py.run("assert C.method() == 'StaticMethod.method()!'", None, Some(&d)).unwrap();
    py.run("assert C().method() == 'StaticMethod.method()!'", None, Some(&d)).unwrap();
}*/

//py_class!(class StaticMethodWithArgs |py| {
//    @staticmethod
//    def method(input: i32) -> PyResult<String> {
//        Ok(format!("0x{:x}", input))
//    }
//});

//#[test]
//fn static_method_with_args() {
//    let gil = Python::acquire_gil();
//    let py = gil.python();

//    assert_eq!(StaticMethodWithArgs::method(py, 1234).unwrap(), "0x4d2");
//    let d = PyDict::new(py);
//    d.set_item(py, "C", py.get_type::<StaticMethodWithArgs>()).unwrap();
//    py.run("assert C.method(1337) == '0x539'", None, Some(&d)).unwrap();
//}

#[py::class]
struct GCIntegration {
    self_ref: RefCell<PyObject>,
    dropped: TestDropCall,
}

#[py::proto]
impl PyGCProtocol for GCIntegration {
    fn __traverse__(&self, py: Python, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&*self.self_ref(py).borrow())
    }

    fn __clear__(&self, py: Python) {
        let old_ref = mem::replace(&mut *self.self_ref(py).borrow_mut(), py.None());
        // Release reference only after the mutable borrow has expired.
        old_ref.release_ref(py);
    }
}

#[test]
fn gc_integration() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called = Arc::new(AtomicBool::new(false));
    let inst = GCIntegration::create_instance(py,
        RefCell::new(py.None()),
        TestDropCall { drop_called: drop_called.clone() }
    ).unwrap();
    *inst.self_ref(py).borrow_mut() = inst.as_object().clone_ref(py);
    inst.release_ref(py);

    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

#[py::class]
pub struct Len {
    l: usize
}

#[py::proto]
impl PyMappingProtocol for Len {
    fn __len__(&self, py: Python) -> PyResult<usize> {
        Ok(*self.l(py))
    }
}

#[test]
fn len() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Len::create_instance(py, 10).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_object().as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_object().as_ptr()), 10);
    }

    let inst = Len::create_instance(py, (isize::MAX as usize) + 1).unwrap();
    py_expect_exception!(py, inst, "len(inst)", OverflowError);
}


/*py_class!(class Iterator |py| {
    data iter: RefCell<Box<iter::Iterator<Item=i32> + Send>>;

    def __iter__(&self) -> PyResult<Iterator> {
        Ok(self.clone_ref(py))
    }

    def __next__(&self) -> PyResult<Option<i32>> {
        Ok(self.iter(py).borrow_mut().next())
    }
});

#[test]
fn iterator() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Iterator::create_instance(py, RefCell::new(Box::new(5..8))).unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}*/

/*
#[py::class]
struct StringMethods {}

#[py::proto]
impl PyObjectProtocol for StringMethods {
    fn __str__(&self, py: Python) -> PyResult<&'static str> {
        Ok("str")
    }

    fn __repr__(&self, py: Python) -> PyResult<&'static str> {
        Ok("repr")
    }

    fn __format__(&self, py: Python, format_spec: &str) -> PyResult<String> {
        Ok(format!("format({})", format_spec))
    }

    //fn __unicode__(&self) -> PyResult<PyString> {
    //    Ok(PyString::new(py, "unicode"))
    //}

    fn __bytes__(&self, py: Python) -> PyResult<PyBytes> {
        Ok(PyBytes::new(py, b"bytes"))
    }
}

#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = StringMethods::create_instance(py).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
}*/
/*
#[test]
fn python3_string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = StringMethods::create_instance(py).unwrap();
    py_assert!(py, obj, "bytes(obj) == b'bytes'");
}*/


#[py::class]
struct Comparisons {
    val: i32,
}

#[py::proto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self, py: Python) -> PyResult<usize> {
        Ok(*self.val(py) as usize)
    }

    fn __bool__(&self, py: Python) -> PyResult<bool> {
        Ok(*self.val(py) != 0)
    }
}


#[test]
fn comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let zero = Comparisons::create_instance(py, 0).unwrap();
    let one = Comparisons::create_instance(py, 1).unwrap();
    let ten = Comparisons::create_instance(py, 10).unwrap();
    let minus_one = Comparisons::create_instance(py, -1).unwrap();
    py_assert!(py, one, "hash(one) == 1");
    py_assert!(py, ten, "hash(ten) == 10");
    py_assert!(py, minus_one, "hash(minus_one) == -2");

    py_assert!(py, one, "bool(one) is True");
    py_assert!(py, zero, "not zero");
}


/*#[py::class]
struct Sequence {}

#[py::proto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self, py: Python) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, py: Python, key: isize) -> PyResult<PyObject> {
        if key == 5 {
            return Err(PyErr::new::<exc::IndexError, NoArgs>(py, NoArgs));
        }
        Ok(key)
    }
}

#[test]
fn sequence() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Sequence::create_instance(py).unwrap();
    py_assert!(py, c, "list(c) == [0, 1, 2, 3, 4]");
    py_assert!(py, c, "c['abc'] == 'abc'");
}*/


#[py::class]
struct Callable {}

#[py::methods]
impl Callable {

    #[__call__]
    fn __call__(&self, py: Python, arg: i32) -> PyResult<i32> {
        Ok(arg * 6)
    }
}

#[test]
fn callable() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Callable::create_instance(py).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = Comparisons::create_instance(py, 0).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[py::class]
struct SetItem {
    key: i32,
    val: i32,
}

#[py::proto]
impl PyMappingProtocol for SetItem {
    fn __setitem__(&self, py: Python, key: i32, val: i32) -> PyResult<()> {
        *self.key_mut(py) = key;
        *self.val_mut(py) = val;
        Ok(())
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = SetItem::create_instance(py, 0, 0).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(*c.key(py), 1);
    assert_eq!(*c.val(py), 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[py::class]
struct DelItem {
    key: i32,
}

#[py::proto]
impl PyMappingProtocol for DelItem {
    fn __delitem__(&self, py: Python, key: i32) -> PyResult<()> {
        *self.key_mut(py) = key;
        Ok(())
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = DelItem::create_instance(py, 0).unwrap();
    py_run!(py, c, "del c[1]");
    assert_eq!(*c.key(py), 1);
    py_expect_exception!(py, c, "c[1] = 2", NotImplementedError);
}

#[py::class]
struct SetDelItem {
    val: Option<i32>,
}

#[py::proto]
impl PyMappingProtocol for SetDelItem {
    fn __setitem__(&self, py: Python, key: i32, val: i32) -> PyResult<()> {
        *self.val_mut(py) = Some(val);
        Ok(())
    }

    fn __delitem__(&self, py: Python, key: i32) -> PyResult<()> {
        *self.val_mut(py) = None;
        Ok(())
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = SetDelItem::create_instance(py, None).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(*c.val(py), Some(2));
    py_run!(py, c, "del c[1]");
    assert_eq!(*c.val(py), None);
}

#[py::class]
struct Reversed {}

#[py::proto]
impl PyMappingProtocol for Reversed{
    fn __reversed__(&self, py: Python) -> PyResult<&'static str> {
        println!("__reversed__");
        Ok("I am reversed")
    }
}

#[test]
fn reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Reversed::create_instance(py).unwrap();
    py_run!(py, c, "assert reversed(c) == 'I am reversed'");
}

/*#[py::class]
struct Contains {}

#[py::proto]
impl PyMappingProtocol for Contains {
    fn __contains__(&self, py: Python, item: i32) -> PyResult<bool> {
        Ok(item >= 0)
    }
}

#[test]
fn contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = Contains::create_instance(py).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_run!(py, c, "assert 'wrong type' not in c");
}*/

/*
py_class!(class UnaryArithmetic |py| {
    def __neg__(&self) -> PyResult<&'static str> {
        Ok("neg")
    }

    def __pos__(&self) -> PyResult<&'static str> {
        Ok("pos")
    }

    def __abs__(&self) -> PyResult<&'static str> {
        Ok("abs")
    }

    def __invert__(&self) -> PyResult<&'static str> {
        Ok("invert")
    }
});

#[test]
fn unary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = UnaryArithmetic::create_instance(py).unwrap();
    py_run!(py, c, "assert -c == 'neg'");
    py_run!(py, c, "assert +c == 'pos'");
    py_run!(py, c, "assert abs(c) == 'abs'");
    py_run!(py, c, "assert ~c == 'invert'");
}*/

/*
#[py::class]
struct BinaryArithmetic {}

#[py::proto]
impl PyObjectProtocol for BinaryArithmetic {
    def __repr__(&self) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[py::proto]
impl PyNumberProtocol for BinaryArithmetic {
    fn __add__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", lhs, rhs))
    }

    fn __sub__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", lhs, rhs))
    }

    fn __mul__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} * {:?}", lhs, rhs))
    }

    fn __lshift__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} << {:?}", lhs, rhs))
    }

    fn __rshift__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} >> {:?}", lhs, rhs))
    }

    fn __and__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} & {:?}", lhs, rhs))
    }

    fn __xor__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} ^ {:?}", lhs, rhs))
    }

    fn __or__(lhs, rhs) -> PyResult<String> {
        Ok(format!("{:?} | {:?}", lhs, rhs))
    }
}

#[test]
fn binary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = BinaryArithmetic::create_instance(py).unwrap();
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
}*/

/*
py_class!(class RichComparisons |py| {
    def __repr__(&self) -> PyResult<&'static str> {
        Ok("RC")
    }

    def __richcmp__(&self, other: &PyObject, op: CompareOp) -> PyResult<String> {
        match op {
            CompareOp::Lt => Ok(format!("{:?} < {:?}", self.as_object(), other)),
            CompareOp::Le => Ok(format!("{:?} <= {:?}", self.as_object(), other)),
            CompareOp::Eq => Ok(format!("{:?} == {:?}", self.as_object(), other)),
            CompareOp::Ne => Ok(format!("{:?} != {:?}", self.as_object(), other)),
            CompareOp::Gt => Ok(format!("{:?} > {:?}", self.as_object(), other)),
            CompareOp::Ge => Ok(format!("{:?} >= {:?}", self.as_object(), other))
        }
    }
});

py_class!(class RichComparisons2 |py| {
    def __repr__(&self) -> PyResult<&'static str> {
        Ok("RC2")
    }

    def __richcmp__(&self, other: &PyObject, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(true.to_py_object(py).into_object()),
            CompareOp::Ne => Ok(false.to_py_object(py).into_object()),
            _ => Ok(py.NotImplemented())
        }
    }
});

#[test]
fn rich_comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = RichComparisons::create_instance(py).unwrap();
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
#[cfg(feature="python3-sys")]
fn rich_comparisons_python_3_type_error() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c2 = RichComparisons2::create_instance(py).unwrap();
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
 */
/*
#[py::class]
struct InPlaceOperations {
    value: u32
}

#[py::proto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("IPO({:?})", self.value(py).get()))
    }
}

#[py::proto]
impl PyNumberProtocol for InPlaceOperations {
    fn __iadd__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() + other);
        Ok(self.clone_ref(py))
    }

    fn __isub__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() - other);
        Ok(self.clone_ref(py))
    }

    fn __imul__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() * other);
        Ok(self.clone_ref(py))
    }

    fn __ilshift__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() << other);
        Ok(self.clone_ref(py))
    }

    fn __irshift__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() >> other);
        Ok(self.clone_ref(py))
    }

    fn __iand__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() & other);
        Ok(self.clone_ref(py))
    }

    fn __ixor__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() ^ other);
        Ok(self.clone_ref(py))
    }

    fn __ior__(&self, py: Python, other: u32) -> PyResult<Self> {
        self.value(py).set(self.value(py).get() | other);
        Ok(self.clone_ref(py))
    }
}

#[test]
fn inplace_operations() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = InPlaceOperations::create_instance(py, Cell::new(0)).unwrap();
    py_run!(py, c, "d = c; c += 1; assert repr(c) == repr(d) == 'IPO(1)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(10)).unwrap();
    py_run!(py, c, "d = c; c -= 1; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(3)).unwrap();
    py_run!(py, c, "d = c; c *= 3; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(3)).unwrap();
    py_run!(py, c, "d = c; c <<= 2; assert repr(c) == repr(d) == 'IPO(12)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(12)).unwrap();
    py_run!(py, c, "d = c; c >>= 2; assert repr(c) == repr(d) == 'IPO(3)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(12)).unwrap();
    py_run!(py, c, "d = c; c &= 10; assert repr(c) == repr(d) == 'IPO(8)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(12)).unwrap();
    py_run!(py, c, "d = c; c |= 3; assert repr(c) == repr(d) == 'IPO(15)'");

    let c = InPlaceOperations::create_instance(py, Cell::new(12)).unwrap();
    py_run!(py, c, "d = c; c ^= 5; assert repr(c) == repr(d) == 'IPO(9)'");
}
*/

#[py::class]
struct ContextManager {
    exit_called: bool
}

#[py::proto]
impl PyContextProtocol for ContextManager {

    fn __enter__(&self, py: Python) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(&self, py: Python,
                ty: Option<PyType>,
                value: Option<PyObject>,
                traceback: Option<PyObject>) -> PyResult<bool> {
        *self.exit_called_mut(py) = true;
        if ty == Some(py.get_type::<exc::ValueError>()) {
            Ok(true)
        } else {
           Ok(false)
        }
    }
}

#[test]
fn context_manager() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = ContextManager::create_instance(py, false).unwrap();
    py_run!(py, c, "with c as x:\n  assert x == 42");
    assert!(*c.exit_called(py));

    *c.exit_called_mut(py) = false;
    py_run!(py, c, "with c as x:\n  raise ValueError");
    assert!(*c.exit_called(py));

    *c.exit_called_mut(py) = false;
    py_expect_exception!(py, c, "with c as x:\n  raise NotImplementedError", NotImplementedError);
    assert!(*c.exit_called(py));
}


#[py::class]
struct ClassWithProperties {
    num: i32
}

#[py::methods]
impl ClassWithProperties {

    fn get_num(&self, py: Python) -> PyResult<i32> {
        Ok(*self.num(py))
    }

    #[getter(DATA)]
    fn get_data(&self, py: Python) -> PyResult<i32> {
        Ok(*self.num(py))
    }
    #[setter(DATA)]
    fn set(&self, py: Python, value: i32) -> PyResult<()> {
        *self.num_mut(py) = value;
        Ok(())
    }
}


#[test]
fn class_with_properties() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = ClassWithProperties::create_instance(py, 10).unwrap();

    py_run!(py, inst, "assert inst.get_num() == 10");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
    py_run!(py, inst, "inst.DATA = 20");
    py_run!(py, inst, "assert inst.get_num() == 20");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");

}
