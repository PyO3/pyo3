#![feature(proc_macro, specialization, const_fn, const_align_of, const_size_of)]
#![allow(dead_code, unused_variables)]

extern crate pyo3;

use pyo3::*;
use std::{isize, iter};
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use pyo3::ffi;


macro_rules! py_run {
    ($py:expr, $val:ident, $code:expr) => {{
        let d = PyDict::new($py);
        d.set_item(stringify!($val), &$val).unwrap();
        $py.run($code, None, Some(d)).map_err(|e| e.print($py)).expect($code);
    }}
}

macro_rules! py_assert {
    ($py:expr, $val:ident, $assertion:expr) => { py_run!($py, $val, concat!("assert ", $assertion)) };
}

macro_rules! py_expect_exception {
    ($py:expr, $val:ident, $code:expr, $err:ident) => {{
        let d = PyDict::new($py);
        d.set_item(stringify!($val), &$val).unwrap();
        let res = $py.run($code, None, Some(d));
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
    assert!(typeobj.call(NoArgs, NoArgs).is_err());

    py_assert!(py, typeobj, "typeobj.__name__ == 'EmptyClass'");
}

/// Line1
///Line2
///  Line3
// this is not doc string
#[py::class]
struct ClassWithDocs { }

#[test]
fn class_with_docstr() {
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let typeobj = py.get_type::<ClassWithDocs>();
        py_run!(py, typeobj, "assert typeobj.__doc__ == 'Line1\\nLine2\\n Line3'");
    }
}

#[py::class(name=CustomName)]
struct EmptyClass2 { }

#[test]
fn custom_class_name() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass2>();
    py_assert!(py, typeobj, "typeobj.__name__ == 'CustomName'");
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
struct EmptyClassWithNew {
    token: PyToken
}

#[py::methods]
impl EmptyClassWithNew {
    #[__new__]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| EmptyClassWithNew{token: t})
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj.call(NoArgs, NoArgs).unwrap().cast_as::<EmptyClassWithNew>().is_ok());
}

#[py::class]
struct NewWithOneArg {
    _data: i32,
    token: PyToken
}

#[py::methods]
impl NewWithOneArg {
    #[new]
    fn __new__(obj: &PyRawObject, arg: i32) -> PyResult<()> {
        obj.init(|t| NewWithOneArg{_data: arg, token: t})
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let wrp = typeobj.call((42,), NoArgs).unwrap();
    let obj = wrp.cast_as::<NewWithOneArg>().unwrap();
    assert_eq!(obj._data, 42);
}

#[py::class]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,

    token: PyToken
}

#[py::methods]
impl NewWithTwoArgs {
    #[new]
    fn __new__(obj: &PyRawObject, arg1: i32, arg2: i32) -> PyResult<()>
    {
        obj.init(|t| NewWithTwoArgs{_data1: arg1, _data2: arg2, token: t})
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let wrp = typeobj.call((10, 20), NoArgs).map_err(|e| e.print(py)).unwrap();
    let obj = wrp.cast_as::<NewWithTwoArgs>().unwrap();
    assert_eq!(obj._data1, 10);
    assert_eq!(obj._data2, 20);
}

#[py::class(freelist=2)]
struct ClassWithFreelist{token: PyToken}

#[test]
fn class_with_freelist() {
    let ptr;
    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let inst = Py::new(py, |t| ClassWithFreelist{token: t}).unwrap();
        let inst2 = Py::new(py, |t| ClassWithFreelist{token: t}).unwrap();
        ptr = inst.as_ptr();
        drop(inst);
    }

    {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let inst3 = Py::new(py, |t| ClassWithFreelist{token: t}).unwrap();
        assert_eq!(ptr, inst3.as_ptr());

        let inst4 = Py::new(py, |t| ClassWithFreelist{token: t}).unwrap();
        assert_ne!(ptr, inst4.as_ptr())
    }
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

#[test]
fn data_is_dropped() {
    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let inst = py.init(|t| DataIsDropped{
            member1: TestDropCall { drop_called: Arc::clone(&drop_called1) },
            member2: TestDropCall { drop_called: Arc::clone(&drop_called2) },
            token: t
        }).unwrap();
        assert!(!drop_called1.load(Ordering::Relaxed));
        assert!(!drop_called2.load(Ordering::Relaxed));
        drop(inst);
    }

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}

#[py::class]
struct ClassWithDrop {
    token: PyToken,
}
impl Drop for ClassWithDrop {
    fn drop(&mut self) {
        unsafe {
            let py = Python::assume_gil_acquired();

            let _empty1 = PyTuple::empty(py);
            let _empty2: PyObject = PyTuple::empty(py).into();
            let _empty3: &PyObjectRef = py.from_owned_ptr(ffi::PyTuple_New(0));
        }
    }
}

// Test behavior of pythonrun::register_pointers + typeob::dealloc
#[test]
fn create_pointers_in_drop() {
    let gil = Python::acquire_gil();

    let ptr;
    let cnt;
    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let empty = PyTuple::empty(py);
        ptr = empty.as_ptr();
        cnt = empty.get_refcnt() - 1;
        let inst = py.init(|t| ClassWithDrop{token: t}).unwrap();
        drop(inst);
    }

    // empty1 and empty2 are still alive (stored in pointers list)
    {
        let _gil = Python::acquire_gil();
        assert_eq!(cnt + 2, unsafe {ffi::Py_REFCNT(ptr)});
    }

    // empty1 and empty2 should be released
    {
        let _gil = Python::acquire_gil();
        assert_eq!(cnt, unsafe {ffi::Py_REFCNT(ptr)});
    }
}

#[py::class]
struct InstanceMethod {
    member: i32,
    token: PyToken
}

#[py::methods]
impl InstanceMethod {
    /// Test method
    fn method(&self) -> PyResult<i32> {
        Ok(self.member)
    }
}

#[test]
fn instance_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.init_ref(|t| InstanceMethod{member: 42, token: t}).unwrap();
    assert!(obj.method().unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item("obj", obj).unwrap();
    py.run("assert obj.method() == 42", None, Some(d)).unwrap();
    py.run("assert obj.method.__doc__ == 'Test method'", None, Some(d)).unwrap();
}

#[py::class]
struct InstanceMethodWithArgs {
    member: i32,
    token: PyToken
}

#[py::methods]
impl InstanceMethodWithArgs {
    fn method(&self, multiplier: i32) -> PyResult<i32> {
        Ok(self.member * multiplier)
    }
}

//#[test]
fn instance_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = py.init_ref(|t| InstanceMethodWithArgs{member: 7, token: t}).unwrap();
    assert!(obj.method(6).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item("obj", obj).unwrap();
    py.run("assert obj.method(3) == 21", None, Some(d)).unwrap();
    py.run("assert obj.method(multiplier=6) == 42", None, Some(d)).unwrap();
}


#[py::class]
struct ClassMethod {token: PyToken}

#[py::methods]
impl ClassMethod {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| ClassMethod{token: t})
    }

    #[classmethod]
    fn method(cls: &PyType) -> PyResult<String> {
        Ok(format!("{}.method()!", cls.name()))
    }
}

#[test]
fn class_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("C", py.get_type::<ClassMethod>()).unwrap();
    py.run("assert C.method() == 'ClassMethod.method()!'", None, Some(d)).unwrap();
    py.run("assert C().method() == 'ClassMethod.method()!'", None, Some(d)).unwrap();
}


#[py::class]
struct ClassMethodWithArgs{token: PyToken}

#[py::methods]
impl ClassMethodWithArgs {
    #[classmethod]
    fn method(cls: &PyType, input: &PyString) -> PyResult<String> {
        Ok(format!("{}.method({})", cls.name(), input))
    }
}

#[test]
fn class_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("C", py.get_type::<ClassMethodWithArgs>()).unwrap();
    py.run("assert C.method('abc') == 'ClassMethodWithArgs.method(abc)'", None, Some(d)).unwrap();
}

#[py::class]
struct StaticMethod {
    token: PyToken
}

#[py::methods]
impl StaticMethod {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| StaticMethod{token: t})
    }

    #[staticmethod]
    fn method(py: Python) -> PyResult<&'static str> {
        Ok("StaticMethod.method()!")
    }
}

#[test]
fn static_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    assert_eq!(StaticMethod::method(py).unwrap(), "StaticMethod.method()!");
    let d = PyDict::new(py);
    d.set_item("C", py.get_type::<StaticMethod>()).unwrap();
    py.run("assert C.method() == 'StaticMethod.method()!'", None, Some(d)).unwrap();
    py.run("assert C().method() == 'StaticMethod.method()!'", None, Some(d)).unwrap();
}

#[py::class]
struct StaticMethodWithArgs{token: PyToken}

#[py::methods]
impl StaticMethodWithArgs {

    #[staticmethod]
    fn method(py: Python, input: i32) -> PyResult<String> {
        Ok(format!("0x{:x}", input))
    }
}

#[test]
fn static_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    assert_eq!(StaticMethodWithArgs::method(py, 1234).unwrap(), "0x4d2");

    let d = PyDict::new(py);
    d.set_item("C", py.get_type::<StaticMethodWithArgs>()).unwrap();
    py.run("assert C.method(1337) == '0x539'", None, Some(d)).unwrap();
}

#[py::class]
struct GCIntegration {
    self_ref: RefCell<PyObject>,
    dropped: TestDropCall,
    token: PyToken,
}

#[py::proto]
impl PyGCProtocol for GCIntegration {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        visit.call(&*self.self_ref.borrow())
    }

    fn __clear__(&mut self) {
        *self.self_ref.borrow_mut() = self.py().None();
    }
}

#[test]
fn gc_integration() {
    let drop_called = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let inst = Py::new_ref(py, |t| GCIntegration{
            self_ref: RefCell::new(py.None()),
            dropped: TestDropCall { drop_called: Arc::clone(&drop_called) },
            token: t}).unwrap();

        *inst.self_ref.borrow_mut() = inst.into();
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

#[py::class(gc)]
struct GCIntegration2 {
    token: PyToken,
}
#[test]
fn gc_integration2() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| GCIntegration2{token: t}).unwrap();
    py_run!(py, inst, "import gc; assert inst in gc.get_objects()");
}

#[py::class(weakref)]
struct WeakRefSupport {
    token: PyToken,
}
#[test]
fn weakref_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| WeakRefSupport{token: t}).unwrap();
    py_run!(py, inst, "import weakref; assert weakref.ref(inst)() is inst");
}

#[py::class]
pub struct Len {
    l: usize,
    token: PyToken,
}

#[py::proto]
impl PyMappingProtocol for Len {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.l)
    }
}

#[test]
fn len() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, |t| Len{l: 10, token: t}).unwrap();
    py_assert!(py, inst, "len(inst) == 10");
    unsafe {
        assert_eq!(ffi::PyObject_Size(inst.as_ptr()), 10);
        assert_eq!(ffi::PyMapping_Size(inst.as_ptr()), 10);
    }

    let inst = Py::new(py, |t| Len{l: (isize::MAX as usize) + 1, token: t}).unwrap();
    py_expect_exception!(py, inst, "len(inst)", OverflowError);
}

#[py::class]
struct Iterator{
    iter: Box<iter::Iterator<Item=i32> + Send>,
    token: PyToken,
}

#[py::proto]
impl PyIterProtocol for Iterator {
    fn __iter__(&mut self) -> PyResult<Py<Iterator>> {
        Ok(self.into())
    }

    fn __next__(&mut self) -> PyResult<Option<i32>> {
        Ok(self.iter.next())
    }
}

#[test]
fn iterator() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = Py::new(py, |t| Iterator{iter: Box::new(5..8), token: t}).unwrap();
    py_assert!(py, inst, "iter(inst) is inst");
    py_assert!(py, inst, "list(inst) == [5, 6, 7]");
}

#[py::class]
struct StringMethods {token: PyToken}

#[py::proto]
impl<'p> PyObjectProtocol<'p> for StringMethods {
    fn __str__(&self) -> PyResult<&'static str> {
        Ok("str")
    }

    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("repr")
    }

    fn __format__(&self, format_spec: String) -> PyResult<String> {
        Ok(format!("format({})", format_spec))
    }

    fn __unicode__(&self) -> PyResult<PyObject> {
        Ok(PyString::new(self.py(), "unicode").into())
    }

    fn __bytes__(&self) -> PyResult<PyObject> {
        Ok(PyBytes::new(self.py(), b"bytes").into())
    }
}

#[cfg(Py_3)]
#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, |t| StringMethods{token: t}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
    py_assert!(py, obj, "bytes(obj) == b'bytes'");
}

#[cfg(not(Py_3))]
#[test]
fn string_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = Py::new(py, |t| StringMethods{token: t}).unwrap();
    py_assert!(py, obj, "str(obj) == 'str'");
    py_assert!(py, obj, "repr(obj) == 'repr'");
    py_assert!(py, obj, "unicode(obj) == 'unicode'");
    py_assert!(py, obj, "'{0:x}'.format(obj) == 'format(x)'");
}


#[py::class]
struct Comparisons {
    val: i32,
    token: PyToken,
}

#[py::proto]
impl PyObjectProtocol for Comparisons {
    fn __hash__(&self) -> PyResult<isize> {
        Ok(self.val as isize)
    }
    fn __bool__(&self) -> PyResult<bool> {
        Ok(self.val != 0)
    }
}


#[test]
fn comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let zero = Py::new(py, |t| Comparisons{val: 0, token: t}).unwrap();
    let one = Py::new(py, |t| Comparisons{val: 1, token: t}).unwrap();
    let ten = Py::new(py, |t| Comparisons{val: 10, token: t}).unwrap();
    let minus_one = Py::new(py, |t| Comparisons{val: -1, token: t}).unwrap();
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

#[py::proto]
impl PySequenceProtocol for Sequence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(5)
    }

    fn __getitem__(&self, key: isize) -> PyResult<isize> {
        if key == 5 {
            return Err(PyErr::new::<exc::IndexError, NoArgs>(NoArgs));
        }
        Ok(key)
    }
}

#[test]
fn sequence() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Sequence{token: t}).unwrap();
    py_assert!(py, c, "list(c) == [0, 1, 2, 3, 4]");
    py_expect_exception!(py, c, "c['abc']", TypeError);
}


#[py::class]
struct Callable {token: PyToken}

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

    let c = py.init(|t| Callable{token: t}).unwrap();
    py_assert!(py, c, "callable(c)");
    py_assert!(py, c, "c(7) == 42");

    let nc = py.init(|t| Comparisons{val: 0, token: t}).unwrap();
    py_assert!(py, nc, "not callable(nc)");
}

#[py::class]
struct SetItem {
    key: i32,
    val: i32,
    token: PyToken,
}

#[py::proto]
impl PyMappingProtocol<'a> for SetItem {
    fn __setitem__(&mut self, key: i32, val: i32) -> PyResult<()> {
        self.key = key;
        self.val = val;
        Ok(())
    }
}

#[test]
fn setitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| SetItem{key: 0, val: 0, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.key, 1);
    assert_eq!(c.val, 2);
    py_expect_exception!(py, c, "del c[1]", NotImplementedError);
}

#[py::class]
struct DelItem {
    key: i32,
    token: PyToken,
}

#[py::proto]
impl PyMappingProtocol<'a> for DelItem {
    fn __delitem__(&mut self, key: i32) -> PyResult<()> {
        self.key = key;
        Ok(())
    }
}

#[test]
fn delitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| DelItem{key:0, token:t}).unwrap();
    py_run!(py, c, "del c[1]");
    assert_eq!(c.key, 1);
    py_expect_exception!(py, c, "c[1] = 2", NotImplementedError);
}

#[py::class]
struct SetDelItem {
    val: Option<i32>,
    token: PyToken,
}

#[py::proto]
impl PyMappingProtocol for SetDelItem {
    fn __setitem__(&mut self, key: i32, val: i32) -> PyResult<()> {
        self.val = Some(val);
        Ok(())
    }

    fn __delitem__(&mut self, key: i32) -> PyResult<()> {
        self.val = None;
        Ok(())
    }
}

#[test]
fn setdelitem() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init_ref(|t| SetDelItem{val: None, token: t}).unwrap();
    py_run!(py, c, "c[1] = 2");
    assert_eq!(c.val, Some(2));
    py_run!(py, c, "del c[1]");
    assert_eq!(c.val, None);
}

#[py::class]
struct Reversed {token: PyToken}

#[py::proto]
impl PyMappingProtocol for Reversed{
    fn __reversed__(&self) -> PyResult<&'static str> {
        Ok("I am reversed")
    }
}

#[test]
fn reversed() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| Reversed{token: t}).unwrap();
    py_run!(py, c, "assert reversed(c) == 'I am reversed'");
}

#[py::class]
struct Contains {token: PyToken}

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

    let c = py.init(|t| Contains{token: t}).unwrap();
    py_run!(py, c, "assert 1 in c");
    py_run!(py, c, "assert -1 not in c");
    py_expect_exception!(py, c, "assert 'wrong type' not in c", TypeError);
}



#[py::class]
struct UnaryArithmetic {token: PyToken}

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

    let c = py.init(|t| UnaryArithmetic{token: t}).unwrap();
    py_run!(py, c, "assert -c == 'neg'");
    py_run!(py, c, "assert +c == 'pos'");
    py_run!(py, c, "assert abs(c) == 'abs'");
    py_run!(py, c, "assert ~c == 'invert'");
}


#[py::class]
struct BinaryArithmetic {
    token: PyToken
}

#[py::proto]
impl PyObjectProtocol for BinaryArithmetic {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("BA")
    }
}

#[py::proto]
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

    let c = py.init(|t| BinaryArithmetic{token: t}).unwrap();
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

#[py::proto]
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
            CompareOp::Ge => Ok(format!("{} >= {:?}", self.__repr__().unwrap(), other))
        }
    }
}

#[py::class]
struct RichComparisons2 {
    py: PyToken
}

#[py::proto]
impl PyObjectProtocol for RichComparisons2 {
    fn __repr__(&self) -> PyResult<&'static str> {
        Ok("RC2")
    }

    fn __richcmp__(&self, other: &'p PyObjectRef, op: CompareOp) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok(true.to_object(self.py())),
            CompareOp::Ne => Ok(false.to_object(self.py())),
            _ => Ok(self.py().NotImplemented())
        }
    }
}

#[test]
fn rich_comparisons() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let c = py.init(|t| RichComparisons{token: t}).unwrap();
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

    let c2 = py.init(|t| RichComparisons2{py: t}).unwrap();
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

#[py::proto]
impl PyObjectProtocol for InPlaceOperations {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("IPO({:?})", self.value))
    }
}

#[py::proto]
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

    let c = py.init(|t| InPlaceOperations{value: 0, token: t}).unwrap();
    py_run!(py, c, "d = c; c += 1; assert repr(c) == repr(d) == 'IPO(1)'");

    let c = py.init(|t| InPlaceOperations{value:10, token: t}).unwrap();
    py_run!(py, c, "d = c; c -= 1; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = py.init(|t| InPlaceOperations{value: 3, token: t}).unwrap();
    py_run!(py, c, "d = c; c *= 3; assert repr(c) == repr(d) == 'IPO(9)'");

    let c = py.init(|t| InPlaceOperations{value: 3, token: t}).unwrap();
    py_run!(py, c, "d = c; c <<= 2; assert repr(c) == repr(d) == 'IPO(12)'");

    let c = py.init(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c >>= 2; assert repr(c) == repr(d) == 'IPO(3)'");

    let c = py.init(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c &= 10; assert repr(c) == repr(d) == 'IPO(8)'");

    let c = py.init(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c |= 3; assert repr(c) == repr(d) == 'IPO(15)'");

    let c = py.init(|t| InPlaceOperations{value: 12, token: t}).unwrap();
    py_run!(py, c, "d = c; c ^= 5; assert repr(c) == repr(d) == 'IPO(9)'");
}

#[py::class]
struct ContextManager {
    exit_called: bool,
    token: PyToken,
}

#[py::proto]
impl<'p> PyContextProtocol<'p> for ContextManager {

    fn __enter__(&mut self) -> PyResult<i32> {
        Ok(42)
    }

    fn __exit__(&mut self,
                ty: Option<&'p PyType>,
                value: Option<&'p PyObjectRef>,
                traceback: Option<&'p PyObjectRef>) -> PyResult<bool> {
        self.exit_called = true;
        if ty == Some(self.py().get_type::<exc::ValueError>()) {
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

    let c = py.init_mut(|t| ContextManager{exit_called: false, token: t}).unwrap();
    py_run!(py, c, "with c as x:\n  assert x == 42");
    assert!(c.exit_called);

    c.exit_called = false;
    py_run!(py, c, "with c as x:\n  raise ValueError");
    assert!(c.exit_called);

    c.exit_called = false;
    py_expect_exception!(
        py, c, "with c as x:\n  raise NotImplementedError", NotImplementedError);
    assert!(c.exit_called);
}

#[py::class]
struct ClassWithProperties {
    num: i32,
    token: PyToken,
}

#[py::methods]
impl ClassWithProperties {

    fn get_num(&self) -> PyResult<i32> {
        Ok(self.num)
    }

    #[getter(DATA)]
    fn get_data(&self) -> PyResult<i32> {
        Ok(self.num)
    }
    #[setter(DATA)]
    fn set_data(&mut self, value: i32) -> PyResult<()> {
        self.num = value;
        Ok(())
    }
}

#[test]
fn class_with_properties() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = py.init(|t| ClassWithProperties{num: 10, token: t}).unwrap();

    py_run!(py, inst, "assert inst.get_num() == 10");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
    py_run!(py, inst, "inst.DATA = 20");
    py_run!(py, inst, "assert inst.get_num() == 20");
    py_run!(py, inst, "assert inst.get_num() == inst.DATA");
}

#[py::class]
struct MethArgs {
    token: PyToken
}

#[py::methods]
impl MethArgs {

    #[args(test="10")]
    fn get_default(&self, test: i32) -> PyResult<i32> {
        Ok(test)
    }
    #[args("*", test=10)]
    fn get_kwarg(&self, test: i32) -> PyResult<i32> {
        Ok(test)
    }
    #[args(args="*", kwargs="**")]
    fn get_kwargs(&self, args: &PyTuple, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
        Ok([args.into(), kwargs.to_object(self.py())].to_object(self.py()))
    }
}

#[test]
fn meth_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = py.init(|t| MethArgs{token: t}).unwrap();

    py_run!(py, inst, "assert inst.get_default() == 10");
    py_run!(py, inst, "assert inst.get_default(100) == 100");
    py_run!(py, inst, "assert inst.get_kwarg() == 10");
    py_run!(py, inst, "assert inst.get_kwarg(100) == 10");
    py_run!(py, inst, "assert inst.get_kwarg(test=100) == 100");
    py_run!(py, inst, "assert inst.get_kwargs() == [(), None]");
    py_run!(py, inst, "assert inst.get_kwargs(1,2,3) == [(1,2,3), None]");
    py_run!(py, inst, "assert inst.get_kwargs(t=1,n=2) == [(), {'t': 1, 'n': 2}]");
    py_run!(py, inst, "assert inst.get_kwargs(1,2,3,t=1,n=2) == [(1,2,3), {'t': 1, 'n': 2}]");
    // py_expect_exception!(py, inst, "inst.get_kwarg(100)", TypeError);
}

#[py::class(subclass)]
struct SubclassAble {}

#[test]
fn subclass() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("SubclassAble", py.get_type::<SubclassAble>()).unwrap();
    py.run("class A(SubclassAble): pass\nassert issubclass(A, SubclassAble)", None, Some(d))
      .map_err(|e| e.print(py))
      .unwrap();
}

#[py::class(dict)]
struct DunderDictSupport {
    token: PyToken,
}

#[test]
fn dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| DunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, "inst.a = 1; assert inst.a == 1");
}

#[py::class(weakref, dict)]
struct WeakRefDunderDictSupport {
    token: PyToken,
}

#[test]
fn weakref_dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| WeakRefDunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1");
}

#[py::class]
struct GetterSetter {
    #[prop(get, set)]
    num: i32,
    token: PyToken
}

#[py::methods]
impl GetterSetter {

    fn get_num2(&self) -> PyResult<i32> {
        Ok(self.num)
    }
}

#[test]
fn getter_setter_autogen() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let inst = py.init(|t| GetterSetter{num: 10, token: t}).unwrap();

    py_run!(py, inst, "assert inst.num == 10");
    py_run!(py, inst, "inst.num = 20; assert inst.num == 20");
}

#[py::class]
struct BaseClass {
    #[prop(get)]
    val1: usize,
}

#[py::methods]
impl BaseClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| BaseClass{val1: 10})
    }
}

#[py::class(base=BaseClass)]
struct SubClass {
    #[prop(get)]
    val2: usize,
}

#[py::methods]
impl SubClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| SubClass{val2: 5})?;
        BaseClass::__new__(obj)
    }
}

#[test]
fn inheritance_with_new_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typebase = py.get_type::<BaseClass>();
    let typeobj = py.get_type::<SubClass>();
    let inst = typeobj.call(NoArgs, NoArgs).unwrap();
    py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
}


#[py::class]
struct BaseClassWithDrop {
    token: PyToken,
    data: Option<Arc<AtomicBool>>,
}

#[py::methods]
impl BaseClassWithDrop {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| BaseClassWithDrop{token: t, data: None})
    }
}

impl Drop for BaseClassWithDrop {
    fn drop(&mut self) {
        if let Some(ref mut data) = self.data {
            data.store(true, Ordering::Relaxed);
        }
    }
}

#[py::class(base=BaseClassWithDrop)]
struct SubClassWithDrop {
    token: PyToken,
    data: Option<Arc<AtomicBool>>,
}

#[py::methods]
impl SubClassWithDrop {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| SubClassWithDrop{token: t, data: None})?;
        BaseClassWithDrop::__new__(obj)
    }
}

impl Drop for SubClassWithDrop {
    fn drop(&mut self) {
        if let Some(ref mut data) = self.data {
            data.store(true, Ordering::Relaxed);
        }
    }
}

#[test]
fn inheritance_with_new_methods_with_drop() {
    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let typebase = py.get_type::<BaseClassWithDrop>();
        let typeobj = py.get_type::<SubClassWithDrop>();
        let inst = typeobj.call(NoArgs, NoArgs).unwrap();

        let obj = SubClassWithDrop::try_from_mut(inst).unwrap();
        obj.data = Some(Arc::clone(&drop_called1));

        let base = obj.get_mut_base();
        base.data = Some(Arc::clone(&drop_called2));
    }

    assert!(drop_called1.load(Ordering::Relaxed));
    assert!(drop_called2.load(Ordering::Relaxed));
}


#[py::class]
struct MutRefArg {
    n: i32,
    token: PyToken,
}

#[py::methods]
impl MutRefArg {

    fn get(&self) -> PyResult<i32> {
        Ok(self.n)
    }
    fn set_other(&self, other: &mut MutRefArg) -> PyResult<()> {
        other.n = 100;
        Ok(())
    }
}

#[test]
fn mut_ref_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst1 = py.init(|t| MutRefArg{token: t, n: 0}).unwrap();
    let inst2 = py.init(|t| MutRefArg{token: t, n: 0}).unwrap();

    let d = PyDict::new(py);
    d.set_item("inst1", &inst1).unwrap();
    d.set_item("inst2", &inst2).unwrap();

    py.run("inst1.set_other(inst2)", None, Some(d)).unwrap();
    assert_eq!(inst2.as_ref(py).n, 100);
}
