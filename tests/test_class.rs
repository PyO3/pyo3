#![feature(proc_macro, specialization)]
#![allow(dead_code, unused_variables)]

extern crate pyo3;

use pyo3::*;
use std::{mem, isize, iter};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use pyo3::ffi;
use pyo3::python::ToPyPointer;


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

    let ty = module.getattr(py, "EmptyClassInModule").unwrap();
    assert_eq!(ty.getattr(py, "__name__").unwrap().extract::<String>(py).unwrap(), "EmptyClassInModule");
    assert_eq!(ty.getattr(py, "__module__").unwrap().extract::<String>(py).unwrap(), "test_module.nested");
}

#[py::class]
struct EmptyClassWithNew {
    token: PyToken
}

#[py::ptr(EmptyClassWithNew)]
struct EmptyClassWithNewPtr(PyPtr);

#[py::methods]
impl EmptyClassWithNew {
    #[__new__]
    fn __new__(cls: &PyType, py: Python) -> PyResult<EmptyClassWithNewPtr> {
        py.with(|t| EmptyClassWithNew{token: t})
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj.call(py, NoArgs, None).unwrap().cast_as::<EmptyClassWithNew>(py).is_ok());
}

#[py::class]
struct NewWithOneArg {
    _data: i32,
    token: PyToken
}

#[py::ptr(NewWithOneArg)]
struct NewWithOneArgPtr(PyPtr);

#[py::methods]
impl NewWithOneArg {
    #[new]
    fn __new__(_cls: &PyType, py: Python, arg: i32) -> PyResult<NewWithOneArgPtr> {
        py.with(|t| NewWithOneArg{_data: arg, token: t})
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let wrp = typeobj.call(py, (42,), None).unwrap();
    let obj = wrp.cast_as::<NewWithOneArg>(py).unwrap();
    assert_eq!(obj._data, 42);
}

#[py::class]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,

    token: PyToken
}

#[py::ptr(NewWithTwoArgs)]
struct NewWithTwoArgsPtr(PyPtr);

#[py::methods]
impl NewWithTwoArgs {
    #[new]
    fn __new__(_cls: &PyType, py: Python, arg1: i32, arg2: i32) -> PyResult<NewWithTwoArgsPtr> {
        py.with(|t| NewWithTwoArgs{_data1: arg1, _data2: arg2, token: t})
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let wrp = typeobj.call(py, (10, 20), None).unwrap();
    let obj = wrp.cast_as::<NewWithTwoArgs>(py).unwrap();
    assert_eq!(obj._data1, 10);
    assert_eq!(obj._data2, 20);
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
    token: PyToken,
}
#[py::ptr(DataIsDropped)]
struct DataIsDroppedPtr(PyPtr);

#[test]
fn data_is_dropped() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));
    let inst = py.with(|t| DataIsDropped{
        member1: TestDropCall { drop_called: drop_called1.clone() },
        member2: TestDropCall { drop_called: drop_called2.clone() },
        token: t
    }).unwrap();
    assert!(drop_called1.load(Ordering::Relaxed) == false);
    assert!(drop_called2.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called1.load(Ordering::Relaxed) == true);
    assert!(drop_called2.load(Ordering::Relaxed) == true);
}


#[py::class]
struct InstanceMethod {
    member: i32,
    token: PyToken
}
#[py::ptr(InstanceMethod)]
struct InstanceMethodPtr(PyPtr);

#[py::methods]
impl InstanceMethod {
    fn method(&self, py: Python) -> PyResult<i32> {
        Ok(self.member)
    }
}

#[test]
fn instance_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.with(|t| InstanceMethod{member: 42, token: t}).unwrap();
    assert!(obj.as_ref(py).method(py).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method() == 42", None, Some(&d)).unwrap();
}

#[py::class]
struct InstanceMethodWithArgs {
    member: i32,
    token: PyToken
}
#[py::ptr(InstanceMethodWithArgs)]
struct InstanceMethodWithArgsPtr(PyPtr);

#[py::methods]
impl InstanceMethodWithArgs {
    fn method(&self, py: Python, multiplier: i32) -> PyResult<i32> {
        Ok(self.member * multiplier)
    }
}

#[test]
fn instance_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.with(|t| InstanceMethodWithArgs{member: 7, token: t}).unwrap();
    assert!(obj.as_ref(py).method(py, 6).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method(3) == 21", None, Some(&d)).unwrap();
    py.run("assert obj.method(multiplier=6) == 42", None, Some(&d)).unwrap();
}

/*
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
}*/

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
struct StaticMethod {
    token: PyToken
}

#[py::ptr(StaticMethod)]
struct StaticMethodPtr(PyPtr);

#[py::methods]
impl StaticMethod {
    #[new]
    fn __new__(cls: &PyType, py: Python) -> PyResult<StaticMethodPtr> {
        py.with(|t| StaticMethod{token: t})
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
    token: PyToken,
}

#[py::ptr(GCIntegration)]
struct GCIntegrationPtr(PyPtr);

#[py::proto]
impl PyGCProtocol for GCIntegration {
    fn __traverse__(&self, py: Python, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&*self.self_ref.borrow())
    }

    fn __clear__(&mut self, py: Python) {
        *self.self_ref.borrow_mut() = py.None();
    }
}

#[test]
fn gc_integration() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called = Arc::new(AtomicBool::new(false));
    let inst = py.with(|t| GCIntegration{
        self_ref: RefCell::new(py.None()),
        dropped: TestDropCall { drop_called: drop_called.clone() },
        token: t}).unwrap();

    *inst.as_mut(py).self_ref.borrow_mut() = inst.clone_ref(py).into();
    drop(inst);

    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

#[py::class]
pub struct Len {
    l: usize,
    token: PyToken,
}

#[py::ptr(Len)]
pub struct LenPtr(PyPtr);

#[py::proto]
impl PyMappingProtocol for Len {
    fn __len__(&self, py: Python) -> PyResult<usize> {
        Ok(self.l)
    }
}

#[test]
fn len() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = py.with(|t| Len{l: 10, token: t}).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 10);
    }

    let inst = py.with(|t| Len{l: (isize::MAX as usize) + 1, token: t}).unwrap();
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

#[py::class]
struct StringMethods {token: PyToken}

#[py::ptr(StringMethods)]
struct StringMethodsPtr(PyPtr);

#[py::proto]
impl<'p> PyObjectProtocol<'p> for StringMethods {
    fn __str__(&self, py: Python) -> PyResult<&'static str> {
        Ok("str")
    }

    fn __repr__(&self, py: Python) -> PyResult<&'static str> {
        Ok("repr")
    }

    fn __format__(&self, py: Python, format_spec: String) -> PyResult<String> {
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

    let obj = py.with(|t| StringMethods{token: t}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
    py_assert!(py, obj, "bytes(obj) == b'bytes'");
}


#[py::class]
struct Comparisons {
    val: i32,
    token: PyToken,
}

#[py::ptr(Comparisons)]
struct ComparisonsPtr(PyPtr);

#[py::proto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self, py: Python) -> PyResult<usize> {
        Ok(self.val as usize)
    }
    fn __bool__(&self, py: Python) -> PyResult<bool> {
        Ok(self.val != 0)
    }
}


#[test]
fn comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let zero = py.with(|t| Comparisons{val: 0, token: t}).unwrap();
    let one = py.with(|t| Comparisons{val: 1, token: t}).unwrap();
    let ten = py.with(|t| Comparisons{val: 10, token: t}).unwrap();
    let minus_one = py.with(|t| Comparisons{val: -1, token: t}).unwrap();
    py_assert!(py, one, "hash(one) == 1");
    py_assert!(py, ten, "hash(ten) == 10");
    py_assert!(py, minus_one, "hash(minus_one) == -2");

    py_assert!(py, one, "bool(one) is True");
    py_assert!(py, zero, "not zero");
}


#[py::class]
struct Sequence {
    token: PyToken
}

#[py::ptr(Sequence)]
struct SequencePtr(PyPtr);

#[py::proto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self, py: Python) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, py: Python, key: isize) -> PyResult<isize> {
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

    let c = py.with(|t| Sequence{token: t}).unwrap();
    py_assert!(py, c, "list(c) == [0, 1, 2, 3, 4]");
    py_expect_exception!(py, c, "c['abc']", TypeError);
}


#[py::class]
struct Callable {token: PyToken}

#[py::ptr(Callable)]
struct CallablePtr(PyPtr);

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

    let c = py.with(|t| Callable{token: t}).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = py.with(|t| Comparisons{val: 0, token: t}).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[py::class]
struct SetItem {
    key: i32,
    val: i32,
    token: PyToken,
}

#[py::ptr(SetItem)]
struct SetItemPtr(PyPtr);

#[py::proto]
impl PyMappingProtocol<'a> for SetItem {
    fn __setitem__(&mut self, py: Python, key: i32, val: i32) -> PyResult<()> {
        self.key = key;
        self.val = val;
        Ok(())
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| SetItem{key: 0, val: 0, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.as_ref(py).key, 1);
    assert_eq!(c.as_ref(py).val, 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[py::class]
struct DelItem {
    key: i32,
    token: PyToken,
}

#[py::ptr(DelItem)]
struct DelItemPtr(PyPtr);

#[py::proto]
impl PyMappingProtocol<'a> for DelItem {
    fn __delitem__(&mut self, py: Python, key: i32) -> PyResult<()> {
        self.key = key;
        Ok(())
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| DelItem{key:0, token:t}).unwrap();
    py_run!(py, c, "del c[1]");
    assert_eq!(c.as_ref(py).key, 1);
    py_expect_exception!(py, c, "c[1] = 2", NotImplementedError);
}

#[py::class]
struct SetDelItem {
    val: Option<i32>,
    token: PyToken,
}

#[py::ptr(SetDelItem)]
struct SetDelItemPtr(PyPtr);

#[py::proto]
impl PyMappingProtocol for SetDelItem {
    fn __setitem__(&mut self, py: Python, key: i32, val: i32) -> PyResult<()> {
        self.val = Some(val);
        Ok(())
    }

    fn __delitem__(&mut self, py: Python, key: i32) -> PyResult<()> {
        self.val = None;
        Ok(())
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| SetDelItem{val: None, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.as_ref(py).val, Some(2));
    py_run!(py, c, "del c[1]");
    assert_eq!(c.as_ref(py).val, None);
}

#[py::class]
struct Reversed {token: PyToken}

#[py::ptr(Reversed)]
struct ReversedPtr(PyPtr);

#[py::proto]
impl PyMappingProtocol for Reversed{
    fn __reversed__(&self, py: Python) -> PyResult<&'static str> {
        Ok("I am reversed")
    }
}

#[test]
fn reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| Reversed{token: t}).unwrap();
    py_run!(py, c, "assert reversed(c) == 'I am reversed'");
}

#[py::class]
struct Contains {token: PyToken}

#[py::ptr(Contains)]
struct ContainsPtr(PyPtr);

#[py::proto]
impl PySequenceProtocol for Contains {
    fn __contains__(&self, py: Python, item: i32) -> PyResult<bool> {
        Ok(item >= 0)
    }
}

#[test]
fn contains() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| Contains{token: t}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", TypeError);
}



#[py::class]
struct UnaryArithmetic {token: PyToken}

#[py::ptr(UnaryArithmetic)]
struct UnaryArithmeticPtr(PyPtr);

#[py::proto]
impl PyNumberProtocol for UnaryArithmetic {

    fn __neg__(&self, py: Python) -> PyResult<&'static str> {
        Ok("neg")
    }

    fn __pos__(&self, py: Python) -> PyResult<&'static str> {
        Ok("pos")
    }

    fn __abs__(&self, py: Python) -> PyResult<&'static str> {
        Ok("abs")
    }

    fn __invert__(&self, py: Python) -> PyResult<&'static str> {
        Ok("invert")
    }
}

#[test]
fn unary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| UnaryArithmetic{token: t}).unwrap();
    py_run!(py, c, "assert -c == 'neg'");
    py_run!(py, c, "assert +c == 'pos'");
    py_run!(py, c, "assert abs(c) == 'abs'");
    py_run!(py, c, "assert ~c == 'invert'");
}


#[py::class]
struct BinaryArithmetic {
    token: PyToken
}

#[py::ptr(BinaryArithmetic)]
struct BinaryArithmeticPtr(PyPtr);

#[py::proto]
impl PyObjectProtocol for BinaryArithmetic {
    fn __repr__(&self, py: Python) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[py::proto]
impl PyNumberProtocol for BinaryArithmetic {
    fn __add__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", self, rhs))
    }

    fn __sub__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", self, rhs))
    }

    fn __mul__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} * {:?}", self, rhs))
    }

    fn __lshift__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} << {:?}", self, rhs))
    }

    fn __rshift__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} >> {:?}", self, rhs))
    }

    fn __and__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} & {:?}", self, rhs))
    }

    fn __xor__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} ^ {:?}", self, rhs))
    }

    fn __or__(&self, py: Python, rhs: &PyObject) -> PyResult<String> {
        Ok(format!("{:?} | {:?}", self, rhs))
    }
}

#[test]
fn binary_arithmetic() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| BinaryArithmetic{token: t}).unwrap();
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


#[py::class]
struct RichComparisons {
    token: PyToken
}

#[py::ptr(RichComparisons)]
struct RichComparisonsPtr(PyPtr);

#[py::proto]
impl PyObjectProtocol for RichComparisons {
    fn __repr__(&self, py: Python) -> PyResult<&'static str> {
        Ok("RC")
    }

    fn __richcmp__(&self, py: Python, other: &PyObject, op: CompareOp) -> PyResult<String> {
        match op {
            CompareOp::Lt => Ok(format!("{} < {:?}", self.__repr__(py).unwrap(), other)),
            CompareOp::Le => Ok(format!("{} <= {:?}", self.__repr__(py).unwrap(), other)),
            CompareOp::Eq => Ok(format!("{} == {:?}", self.__repr__(py).unwrap(), other)),
            CompareOp::Ne => Ok(format!("{} != {:?}", self.__repr__(py).unwrap(), other)),
            CompareOp::Gt => Ok(format!("{} > {:?}", self.__repr__(py).unwrap(), other)),
            CompareOp::Ge => Ok(format!("{} >= {:?}", self.__repr__(py).unwrap(), other))
        }
    }
}

#[py::class]
struct RichComparisons2 {
    py: PyToken
}

#[py::ptr(RichComparisons2)]
struct RichComparisons2Ptr(PyPtr);

#[py::proto]
impl PyObjectProtocol for RichComparisons2 {
    fn __repr__(&self, py: Python) -> PyResult<&'static str> {
        Ok("RC2")
    }

    fn __richcmp__(&self, py: Python,
                   other: &'p PyObject, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(true.to_object(py)),
            CompareOp::Ne => Ok(false.to_object(py)),
            _ => Ok(py.NotImplemented())
        }
    }
}

#[test]
fn rich_comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| RichComparisons{token: t}).unwrap();
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

    let c2 = py.with(|t| RichComparisons2{py: t}).unwrap();
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

#[py::class]
struct InPlaceOperations {
    value: u32,
    token: PyToken,
}

#[py::ptr(InPlaceOperations)]
struct InPlaceOperationsPtr(PyPtr);

#[py::proto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!("IPO({:?})", self.value))
    }
}

#[py::proto]
impl PyNumberProtocol for InPlaceOperations {
    fn __iadd__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value + other;
        Ok(())
    }

    fn __isub__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value - other;
        Ok(())
    }

    fn __imul__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value * other;
        Ok(())
    }

    fn __ilshift__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value << other;
        Ok(())
    }

    fn __irshift__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value >> other;
        Ok(())
    }

    fn __iand__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value & other;
        Ok(())
    }

    fn __ixor__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value ^ other;
        Ok(())
    }

    fn __ior__(&mut self, py: Python, other: u32) -> PyResult<()> {
        self.value = self.value | other;
        Ok(())
    }
}

#[test]
fn inplace_operations() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.with(|t| InPlaceOperations{value: 0, token: t}).unwrap();
    py_run!(py, c, "d = c; c += 1; assert repr(c) == repr(d) == 'IPO(1)'");

    let c = py.with(|t| InPlaceOperations{value:10, token: t}).unwrap();
    py_run!(py, c, "d = c; c -= 1; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = py.with(|t| InPlaceOperations{value: 3, token: t}).unwrap();
    py_run!(py, c, "d = c; c *= 3; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = py.with(|t| InPlaceOperations{value: 3, token: t}).unwrap();
    py_run!(py, c, "d = c; c <<= 2; assert repr(c) == repr(d) == 'IPO(12)'");

    let c = py.with(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c >>= 2; assert repr(c) == repr(d) == 'IPO(3)'");

    let c = py.with(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c &= 10; assert repr(c) == repr(d) == 'IPO(8)'");

    let c = py.with(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c |= 3; assert repr(c) == repr(d) == 'IPO(15)'");

    let c = py.with(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c ^= 5; assert repr(c) == repr(d) == 'IPO(9)'");
}

#[py::class]
struct ContextManager {
    exit_called: bool,
    token: PyToken,
}

#[py::ptr(ContextManager)]
struct ContextManagerPtr(PyPtr);

#[py::proto]
impl<'p> PyContextProtocol<'p> for ContextManager {

    fn __enter__(&mut self, py: Python) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(&mut self, py: Python,
                ty: Option<PyType>,
                value: Option<PyObject>,
                traceback: Option<PyObject>) -> PyResult<bool> {
        self.exit_called = true;
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

    let c = py.with(|t| ContextManager{exit_called: false, token: t}).unwrap();
    py_run!(py, c, "with c as x:\n  assert x == 42");
    assert!(c.as_ref(py).exit_called);

    c.as_mut(py).exit_called = false;
    py_run!(py, c, "with c as x:\n  raise ValueError");
    assert!(c.as_ref(py).exit_called);

    c.as_mut(py).exit_called = false;
    py_expect_exception!(
        py, c, "with c as x:\n  raise NotImplementedError",
        NotImplementedError);
    assert!(c.as_ref(py).exit_called);
}

#[py::class]
struct ClassWithProperties {
    num: i32,
    token: PyToken,
}

#[py::ptr(ClassWithProperties)]
struct ClassWithPropertiesPtr(PyPtr);

#[py::methods]
impl ClassWithProperties {

    fn get_num(&self, py: Python) -> PyResult<i32> {
        Ok(self.num)
    }

    #[getter(DATA)]
    fn get_data(&self, py: Python) -> PyResult<i32> {
        Ok(self.num)
    }
    #[setter(DATA)]
    fn set_data(&mut self, py: Python, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
    }
}


#[test]
fn class_with_properties() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = py.with(|t| ClassWithProperties{num: 10, token: t}).unwrap();

    py_run!(py, inst, "assert inst.get_num() == 10");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
    py_run!(py, inst, "inst.DATA = 20");
    py_run!(py, inst, "assert inst.get_num() == 20");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");

}
