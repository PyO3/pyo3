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
        d.set_item(stringify!($val), &$val).unwrap();
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
    assert!(typeobj.call(NoArgs, None).is_err());

    py_assert!(py, typeobj, "typeobj.__name__ == 'EmptyClass'");
}

#[py::class]
struct EmptyClassInModule { }

#[test]
fn empty_class_in_module() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let module = PyModule::new(py, "test_module.nested").unwrap();
    module.add_class::<EmptyClassInModule>().unwrap();

    let ty = module.getattr("EmptyClassInModule").unwrap();
    assert_eq!(ty.getattr("__name__").unwrap().extract::<String>().unwrap(), "EmptyClassInModule");
    assert_eq!(ty.getattr("__module__").unwrap().extract::<String>().unwrap(), "test_module.nested");
}

#[py::class]
struct EmptyClassWithNew { }

#[py::methods]
impl EmptyClassWithNew {
    #[__new__]
    fn __new__(cls: &PyType) -> PyResult<EmptyClassWithNew> {
        Ok(EmptyClassWithNew{})
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj.call(NoArgs, None).unwrap().cast_into::<EmptyClassWithNew>().is_ok());
}

#[py::class]
struct NewWithOneArg {
    _data: i32,
}
#[py::methods]
impl NewWithOneArg {
    #[new]
    fn __new__(_cls: &PyType, arg: i32) -> PyResult<NewWithOneArg> {
        Ok(NewWithOneArg{_data: arg})
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let obj = typeobj.call((42,), None).unwrap().cast_into::<NewWithOneArg>().unwrap();
    assert_eq!(obj._data, 42);
}

#[py::class]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,
}

#[py::methods]
impl NewWithTwoArgs {
    #[new]
    fn __new__(_cls: &PyType, arg1: i32, arg2: i32) -> PyResult<NewWithTwoArgs> {
        Ok(NewWithTwoArgs{_data1: arg1, _data2: arg2})
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let obj = typeobj.call((10, 20), None).unwrap().cast_into::<NewWithTwoArgs>().unwrap();
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
}

//#[test]
fn data_is_dropped() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));
    let inst = DataIsDropped{
        member1: TestDropCall { drop_called: drop_called1.clone() },
        member2: TestDropCall { drop_called: drop_called2.clone() }}.into_object(py);
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
    fn method(&self) -> PyResult<i32> {
        Ok(self.member)
    }
}

#[test]
fn instance_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.init(InstanceMethod{member: 42});
    assert!(obj.method().unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item("obj", obj).unwrap();
    py.run("assert obj.method() == 42", None, Some(&d)).unwrap();
}

#[py::class]
struct InstanceMethodWithArgs {
    member: i32
}
#[py::methods]
impl InstanceMethodWithArgs {
    fn method(&self, multiplier: i32) -> PyResult<i32> {
        Ok(self.member * multiplier)
    }
}

#[test]
fn instance_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.init(InstanceMethodWithArgs{member: 7});
    assert!(obj.method(6).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item("obj", obj).unwrap();
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

#[py::class]
struct StringMethods {}

#[py::proto]
impl PyObjectProtocol for StringMethods {
    fn __str__(&self) -> PyResult<&'static str> {
        Ok("str")
    }

    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("repr")
    }

    fn __format__(&self, format_spec: String) -> PyResult<String> {
        Ok(format!("format({})", format_spec))
    }

    //fn __unicode__(&self) -> PyResult<PyString> {
    //    Ok(PyString::new(py, "unicode"))
    //}

    fn __bytes__(&self) -> PyResult<PyBytes> {
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
    py_assert!(py, obj, "bytes(obj) == b'bytes'");
}


#[py::class]
struct Comparisons {
    val: i32,
}

#[py::proto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self) -> PyResult<usize> {
        Ok(*self.val(py) as usize)
    }

    fn __bool__(&self) -> PyResult<bool> {
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


#[py::class]
struct Sequence {}

#[py::proto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, key: isize) -> PyResult<isize> {
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
    py_expect_exception!(py, c, "c['abc']", TypeError);
}


#[py::class]
struct Callable {}

#[py::methods]
impl Callable {

    #[__call__]
    fn __call__(&self, arg: i32) -> PyResult<i32> {
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
impl PyMappingProtocol<'a> for SetItem {
    fn __setitem__(&self, key: i32, val: i32) -> PyResult<()> {
        *self.key_mut(py) = key;
        *self.val_mut(py) = val;
        Ok(())
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = SetItem::create_instance(0, 0).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.key, 1);
    assert_eq!(c.val, 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[py::class]
struct DelItem {
    key: i32,
}

#[py::proto]
impl PyMappingProtocol<'a> for DelItem {
    fn __delitem__(&self, key: i32) -> PyResult<()> {
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
    fn __setitem__(&self, key: i32, val: i32) -> PyResult<()> {
        *self.val_mut(py) = Some(val);
        Ok(())
    }

    fn __delitem__(&self, key: i32) -> PyResult<()> {
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
    fn __reversed__(&self) -> PyResult<&'static str> {
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

#[py::class]
struct Contains {}

#[py::proto]
impl PySequenceProtocol for Contains {
    fn __contains__(&self, item: i32) -> PyResult<bool> {
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
    py_expect_exception!(py, c, "assert 'wrong type' not in c", TypeError);
}



#[py::class]
struct UnaryArithmetic {}

#[py::proto]
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

    let c = UnaryArithmetic::create_instance(py).unwrap();
    py_run!(py, c, "assert -c == 'neg'");
    py_run!(py, c, "assert +c == 'pos'");
    py_run!(py, c, "assert abs(c) == 'abs'");
    py_run!(py, c, "assert ~c == 'invert'");
}


#[py::class]
struct BinaryArithmetic {}

#[py::proto]
impl PyObjectProtocol for BinaryArithmetic {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[py::proto]
impl PyNumberProtocol for BinaryArithmetic {
    fn __add__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} + {:?}", self.as_object(), rhs))
    }

    fn __sub__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} - {:?}", self.as_object(), rhs))
    }

    fn __mul__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} * {:?}", self.as_object(), rhs))
    }

    fn __lshift__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} << {:?}", self.as_object(), rhs))
    }

    fn __rshift__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} >> {:?}", self.as_object(), rhs))
    }

    fn __and__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} & {:?}", self.as_object(), rhs))
    }

    fn __xor__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} ^ {:?}", self.as_object(), rhs))
    }

    fn __or__(&self, rhs: PyObject) -> PyResult<String> {
        Ok(format!("{:?} | {:?}", self.as_object(), rhs))
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
}


#[py::class]
struct RichComparisons {}

#[py::proto]
impl PyObjectProtocol for RichComparisons {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC")
    }

    fn __richcmp__(&self, other: PyObject, op: CompareOp) -> PyResult<String> {
        match op {
            CompareOp::Lt => Ok(format!("{:?} < {:?}", self.as_object(), other)),
            CompareOp::Le => Ok(format!("{:?} <= {:?}", self.as_object(), other)),
            CompareOp::Eq => Ok(format!("{:?} == {:?}", self.as_object(), other)),
            CompareOp::Ne => Ok(format!("{:?} != {:?}", self.as_object(), other)),
            CompareOp::Gt => Ok(format!("{:?} > {:?}", self.as_object(), other)),
            CompareOp::Ge => Ok(format!("{:?} >= {:?}", self.as_object(), other))
        }
    }
}

#[py::class]
struct RichComparisons2 {}

#[py::proto]
impl PyObjectProtocol for RichComparisons2 {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC2")
    }

    fn __richcmp__(&self, other: PyObject, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(true.to_py_object(py).into_object()),
            CompareOp::Ne => Ok(false.to_py_object(py).into_object()),
            _ => Ok(py.NotImplemented())
        }
    }
}

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

#[py::class]
struct InPlaceOperations {
    value: u32
}

#[py::proto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("IPO({:?})", self.value(py)))
    }
}

#[py::proto]
impl PyNumberProtocol for InPlaceOperations {
    fn __iadd__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) + other;
        Ok(self.clone_ref(py))
    }

    fn __isub__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) - other;
        Ok(self.clone_ref(py))
    }

    fn __imul__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) * other;
        Ok(self.clone_ref(py))
    }

    fn __ilshift__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) << other;
        Ok(self.clone_ref(py))
    }

    fn __irshift__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) >> other;
        Ok(self.clone_ref(py))
    }

    fn __iand__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) & other;
        Ok(self.clone_ref(py))
    }

    fn __ixor__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) ^ other;
        Ok(self.clone_ref(py))
    }

    fn __ior__(&self, other: u32) -> PyResult<Self> {
        *self.value_mut(py) = *self.value(py) | other;
        Ok(self.clone_ref(py))
    }
}

#[test]
fn inplace_operations() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = InPlaceOperations::create_instance(py, 0).unwrap();
    py_run!(py, c, "d = c; c += 1; assert repr(c) == repr(d) == 'IPO(1)'");

    let c = InPlaceOperations::create_instance(py, 10).unwrap();
    py_run!(py, c, "d = c; c -= 1; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = InPlaceOperations::create_instance(py, 3).unwrap();
    py_run!(py, c, "d = c; c *= 3; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = InPlaceOperations::create_instance(py, 3).unwrap();
    py_run!(py, c, "d = c; c <<= 2; assert repr(c) == repr(d) == 'IPO(12)'");

    let c = InPlaceOperations::create_instance(py, 12).unwrap();
    py_run!(py, c, "d = c; c >>= 2; assert repr(c) == repr(d) == 'IPO(3)'");

    let c = InPlaceOperations::create_instance(py, 12).unwrap();
    py_run!(py, c, "d = c; c &= 10; assert repr(c) == repr(d) == 'IPO(8)'");

    let c = InPlaceOperations::create_instance(py, 12).unwrap();
    py_run!(py, c, "d = c; c |= 3; assert repr(c) == repr(d) == 'IPO(15)'");

    let c = InPlaceOperations::create_instance(py, 12).unwrap();
    py_run!(py, c, "d = c; c ^= 5; assert repr(c) == repr(d) == 'IPO(9)'");
}

#[py::class]
struct ContextManager {
    exit_called: bool
}

#[py::proto]
impl<'p> PyContextProtocol<'p> for ContextManager {

    fn __enter__(&self) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(&mut self,
                ty: Option<&'p PyType>,
                value: Option<&'p PyObject>,
                traceback: Option<&'p PyObject>) -> PyResult<bool> {
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

    fn get_num(&self) -> PyResult<i32> {
        Ok(*self.num(py))
    }

    #[getter(DATA)]
    fn get_data(&self) -> PyResult<i32> {
        Ok(*self.num(py))
    }
    #[setter(DATA)]
    fn set(&self, value: i32) -> PyResult<()> {
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