#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::{IntoPyDict, PyDict, PyList, PySet, PyString, PyTuple, PyType};

#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct InstanceMethod {
    member: i32,
}

#[pymethods]
impl InstanceMethod {
    /// Test method
    fn method(&self) -> i32 {
        self.member
    }

    // Checks that &Self works
    fn add_other(&self, other: &Self) -> i32 {
        self.member + other.member
    }
}

#[test]
fn instance_method() {
    Python::with_gil(|py| {
        let obj = Bound::new(py, InstanceMethod { member: 42 }).unwrap();
        let obj_ref = obj.borrow();
        assert_eq!(obj_ref.method(), 42);
        py_assert!(py, obj, "obj.method() == 42");
        py_assert!(py, obj, "obj.add_other(obj) == 84");
        py_assert!(py, obj, "obj.method.__doc__ == 'Test method'");
    });
}

#[pyclass]
struct InstanceMethodWithArgs {
    member: i32,
}

#[pymethods]
impl InstanceMethodWithArgs {
    fn method(&self, multiplier: i32) -> i32 {
        self.member * multiplier
    }
}

#[test]
fn instance_method_with_args() {
    Python::with_gil(|py| {
        let obj = Bound::new(py, InstanceMethodWithArgs { member: 7 }).unwrap();
        let obj_ref = obj.borrow();
        assert_eq!(obj_ref.method(6), 42);
        py_assert!(py, obj, "obj.method(3) == 21");
        py_assert!(py, obj, "obj.method(multiplier=6) == 42");
    });
}

#[pyclass]
struct ClassMethod {}

#[pymethods]
impl ClassMethod {
    #[new]
    fn new() -> Self {
        ClassMethod {}
    }

    #[classmethod]
    /// Test class method.
    fn method(cls: &Bound<'_, PyType>) -> PyResult<String> {
        Ok(format!("{}.method()!", cls.qualname()?))
    }

    #[classmethod]
    /// Test class method.
    #[cfg(feature = "gil-refs")]
    fn method_gil_ref(cls: &PyType) -> PyResult<String> {
        Ok(format!("{}.method()!", cls.qualname()?))
    }

    #[classmethod]
    fn method_owned(cls: Py<PyType>) -> PyResult<String> {
        let qualname = Python::with_gil(|gil| cls.bind(gil).qualname())?;
        Ok(format!("{}.method_owned()!", qualname))
    }
}

#[test]
fn class_method() {
    Python::with_gil(|py| {
        let d = [("C", py.get_type_bound::<ClassMethod>())].into_py_dict_bound(py);
        py_assert!(py, *d, "C.method() == 'ClassMethod.method()!'");
        py_assert!(py, *d, "C().method() == 'ClassMethod.method()!'");
        py_assert!(
            py,
            *d,
            "C().method_owned() == 'ClassMethod.method_owned()!'"
        );
        py_assert!(py, *d, "C.method.__doc__ == 'Test class method.'");
        py_assert!(py, *d, "C().method.__doc__ == 'Test class method.'");
    });
}

#[pyclass]
struct ClassMethodWithArgs {}

#[pymethods]
impl ClassMethodWithArgs {
    #[classmethod]
    fn method(cls: &Bound<'_, PyType>, input: &Bound<'_, PyString>) -> PyResult<String> {
        Ok(format!("{}.method({})", cls.qualname()?, input))
    }
}

#[test]
fn class_method_with_args() {
    Python::with_gil(|py| {
        let d = [("C", py.get_type_bound::<ClassMethodWithArgs>())].into_py_dict_bound(py);
        py_assert!(
            py,
            *d,
            "C.method('abc') == 'ClassMethodWithArgs.method(abc)'"
        );
    });
}

#[pyclass]
struct StaticMethod {}

#[pymethods]
impl StaticMethod {
    #[new]
    fn new() -> Self {
        StaticMethod {}
    }

    #[staticmethod]
    /// Test static method.
    fn method(_py: Python<'_>) -> &'static str {
        "StaticMethod.method()!"
    }
}

#[test]
fn static_method() {
    Python::with_gil(|py| {
        assert_eq!(StaticMethod::method(py), "StaticMethod.method()!");

        let d = [("C", py.get_type_bound::<StaticMethod>())].into_py_dict_bound(py);
        py_assert!(py, *d, "C.method() == 'StaticMethod.method()!'");
        py_assert!(py, *d, "C().method() == 'StaticMethod.method()!'");
        py_assert!(py, *d, "C.method.__doc__ == 'Test static method.'");
        py_assert!(py, *d, "C().method.__doc__ == 'Test static method.'");
    });
}

#[pyclass]
struct StaticMethodWithArgs {}

#[pymethods]
impl StaticMethodWithArgs {
    #[staticmethod]
    fn method(_py: Python<'_>, input: i32) -> String {
        format!("0x{:x}", input)
    }
}

#[test]
fn static_method_with_args() {
    Python::with_gil(|py| {
        assert_eq!(StaticMethodWithArgs::method(py, 1234), "0x4d2");

        let d = [("C", py.get_type_bound::<StaticMethodWithArgs>())].into_py_dict_bound(py);
        py_assert!(py, *d, "C.method(1337) == '0x539'");
    });
}

#[pyclass]
struct MethSignature {}

#[pymethods]
impl MethSignature {
    #[pyo3(signature = (test = None))]
    fn get_optional(&self, test: Option<i32>) -> i32 {
        test.unwrap_or(10)
    }
    #[pyo3(signature = (test = None))]
    fn get_optional2(&self, test: Option<i32>) -> Option<i32> {
        test
    }
    fn get_optional_positional(
        &self,
        _t1: Option<i32>,
        t2: Option<i32>,
        _t3: Option<i32>,
    ) -> Option<i32> {
        t2
    }

    #[pyo3(signature = (test = 10))]
    fn get_default(&self, test: i32) -> i32 {
        test
    }
    #[pyo3(signature = (*, test = 10))]
    fn get_kwarg(&self, test: i32) -> i32 {
        test
    }
    #[pyo3(signature = (*args, **kwargs))]
    fn get_kwargs(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyObject {
        [args.to_object(py), kwargs.to_object(py)].to_object(py)
    }

    #[pyo3(signature = (a, *args, **kwargs))]
    fn get_pos_arg_kw(
        &self,
        py: Python<'_>,
        a: i32,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyObject {
        [a.to_object(py), args.to_object(py), kwargs.to_object(py)].to_object(py)
    }

    #[pyo3(signature = (a, b, /))]
    fn get_pos_only(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (a, /, b))]
    fn get_pos_only_and_pos(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (a, /, b, c = 5))]
    fn get_pos_only_and_pos_and_kw(&self, a: i32, b: i32, c: i32) -> i32 {
        a + b + c
    }

    #[pyo3(signature = (a, /, *, b))]
    fn get_pos_only_and_kw_only(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (a, /, *, b = 3))]
    fn get_pos_only_and_kw_only_with_default(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (a, /, b, *, c, d = 5))]
    fn get_all_arg_types_together(&self, a: i32, b: i32, c: i32, d: i32) -> i32 {
        a + b + c + d
    }

    #[pyo3(signature = (a, /, *args))]
    fn get_pos_only_with_varargs(&self, a: i32, args: Vec<i32>) -> i32 {
        a + args.iter().sum::<i32>()
    }

    #[pyo3(signature = (a, /, **kwargs))]
    fn get_pos_only_with_kwargs(
        &self,
        py: Python<'_>,
        a: i32,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyObject {
        [a.to_object(py), kwargs.to_object(py)].to_object(py)
    }

    #[pyo3(signature = (a=0, /, **kwargs))]
    fn get_optional_pos_only_with_kwargs(
        &self,
        py: Python<'_>,
        a: i32,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyObject {
        [a.to_object(py), kwargs.to_object(py)].to_object(py)
    }

    #[pyo3(signature = (*, a = 2, b = 3))]
    fn get_kwargs_only_with_defaults(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (*, a, b))]
    fn get_kwargs_only(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (*, a = 1, b))]
    fn get_kwargs_only_with_some_default(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[pyo3(signature = (*args, a))]
    fn get_args_and_required_keyword(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        a: i32,
    ) -> PyObject {
        (args, a).to_object(py)
    }

    #[pyo3(signature = (a, b = 2, *, c = 3))]
    fn get_pos_arg_kw_sep1(&self, a: i32, b: i32, c: i32) -> i32 {
        a + b + c
    }

    #[pyo3(signature = (a, *, b = 2, c = 3))]
    fn get_pos_arg_kw_sep2(&self, a: i32, b: i32, c: i32) -> i32 {
        a + b + c
    }

    #[pyo3(signature = (a, **kwargs))]
    fn get_pos_kw(&self, py: Python<'_>, a: i32, kwargs: Option<&Bound<'_, PyDict>>) -> PyObject {
        [a.to_object(py), kwargs.to_object(py)].to_object(py)
    }

    // "args" can be anything that can be extracted from PyTuple
    #[pyo3(signature = (*args))]
    fn args_as_vec(&self, args: Vec<i32>) -> i32 {
        args.iter().sum()
    }
}

#[test]
fn meth_signature() {
    Python::with_gil(|py| {
        let inst = Py::new(py, MethSignature {}).unwrap();

        py_run!(py, inst, "assert inst.get_optional() == 10");
        py_run!(py, inst, "assert inst.get_optional(100) == 100");
        py_run!(py, inst, "assert inst.get_optional2() == None");
        py_run!(py, inst, "assert inst.get_optional2(100) == 100");
        py_run!(
            py,
            inst,
            "assert inst.get_optional_positional(1, 2, 3) == 2"
        );
        py_run!(py, inst, "assert inst.get_optional_positional(1) == None");
        py_run!(py, inst, "assert inst.get_default() == 10");
        py_run!(py, inst, "assert inst.get_default(100) == 100");
        py_run!(py, inst, "assert inst.get_kwarg() == 10");
        py_expect_exception!(py, inst, "inst.get_kwarg(100)", PyTypeError);
        py_run!(py, inst, "assert inst.get_kwarg(test=100) == 100");
        py_run!(py, inst, "assert inst.get_kwargs() == [(), None]");
        py_run!(py, inst, "assert inst.get_kwargs(1,2,3) == [(1,2,3), None]");
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs(t=1,n=2) == [(), {'t': 1, 'n': 2}]"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs(1,2,3,t=1,n=2) == [(1,2,3), {'t': 1, 'n': 2}]"
        );

        py_run!(py, inst, "assert inst.get_pos_arg_kw(1) == [1, (), None]");
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw(1, 2, 3) == [1, (2, 3), None]"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw(1, b=2) == [1, (), {'b': 2}]"
        );
        py_run!(py, inst, "assert inst.get_pos_arg_kw(a=1) == [1, (), None]");
        py_expect_exception!(py, inst, "inst.get_pos_arg_kw()", PyTypeError);
        py_expect_exception!(py, inst, "inst.get_pos_arg_kw(1, a=1)", PyTypeError);
        py_expect_exception!(py, inst, "inst.get_pos_arg_kw(b=2)", PyTypeError);

        py_run!(py, inst, "assert inst.get_pos_only(10, 11) == 21");
        py_expect_exception!(py, inst, "inst.get_pos_only(10, b = 11)", PyTypeError);
        py_expect_exception!(py, inst, "inst.get_pos_only(a = 10, b = 11)", PyTypeError);

        py_run!(py, inst, "assert inst.get_pos_only_and_pos(10, 11) == 21");
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_pos(10, b = 11) == 21"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_pos(a = 10, b = 11)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_pos_and_kw(10, 11) == 26"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_pos_and_kw(10, b = 11) == 26"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_pos_and_kw(10, 11, c = 0) == 21"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_pos_and_kw(10, b = 11, c = 0) == 21"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_pos_and_kw(a = 10, b = 11)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_kw_only(10, b = 11) == 21"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_kw_only(10, 11)",
            PyTypeError
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_kw_only(a = 10, b = 11)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_kw_only_with_default(10) == 13"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_and_kw_only_with_default(10, b = 11) == 21"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_kw_only_with_default(10, 11)",
            PyTypeError
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_and_kw_only_with_default(a = 10, b = 11)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_all_arg_types_together(10, 10, c = 10) == 35"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_all_arg_types_together(10, 10, c = 10, d = 10) == 40"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_all_arg_types_together(10, b = 10, c = 10, d = 10) == 40"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_all_arg_types_together(10, 10, 10)",
            PyTypeError
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_all_arg_types_together(a = 10, b = 10, c = 10)",
            PyTypeError
        );

        py_run!(py, inst, "assert inst.get_pos_only_with_varargs(10) == 10");
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_with_varargs(10, 10) == 20"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_with_varargs(10, 10, 10, 10, 10) == 50"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_with_varargs(a = 10)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_with_kwargs(10) == [10, None]"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_only_with_kwargs(10, b = 10) == [10, {'b': 10}]"
        );
        py_run!(
        py,
        inst,
        "assert inst.get_pos_only_with_kwargs(10, b = 10, c = 10, d = 10, e = 10) == [10, {'b': 10, 'c': 10, 'd': 10, 'e': 10}]"
    );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_with_kwargs(a = 10)",
            PyTypeError
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_pos_only_with_kwargs(a = 10, b = 10)",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_optional_pos_only_with_kwargs() == [0, None]"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_optional_pos_only_with_kwargs(10) == [10, None]"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_optional_pos_only_with_kwargs(a=10) == [0, {'a': 10}]"
        );

        py_run!(py, inst, "assert inst.get_kwargs_only_with_defaults() == 5");
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_defaults(a = 8) == 11"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_defaults(b = 8) == 10"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_defaults(a = 1, b = 1) == 2"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_defaults(b = 1, a = 1) == 2"
        );

        py_run!(py, inst, "assert inst.get_kwargs_only(a = 1, b = 1) == 2");
        py_run!(py, inst, "assert inst.get_kwargs_only(b = 1, a = 1) == 2");

        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_some_default(a = 2, b = 1) == 3"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_some_default(b = 1) == 2"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_kwargs_only_with_some_default(b = 1, a = 2) == 3"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_kwargs_only_with_some_default()",
            PyTypeError
        );

        py_run!(
            py,
            inst,
            "assert inst.get_args_and_required_keyword(1, 2, a=3) == ((1, 2), 3)"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_args_and_required_keyword(a=1) == ((), 1)"
        );
        py_expect_exception!(
            py,
            inst,
            "inst.get_args_and_required_keyword()",
            PyTypeError
        );

        py_run!(py, inst, "assert inst.get_pos_arg_kw_sep1(1) == 6");
        py_run!(py, inst, "assert inst.get_pos_arg_kw_sep1(1, 2) == 6");
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw_sep1(1, 2, c=13) == 16"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw_sep1(a=1, b=2, c=13) == 16"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw_sep1(b=2, c=13, a=1) == 16"
        );
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw_sep1(c=13, b=2, a=1) == 16"
        );
        py_expect_exception!(py, inst, "inst.get_pos_arg_kw_sep1(1, 2, 3)", PyTypeError);

        py_run!(py, inst, "assert inst.get_pos_arg_kw_sep2(1) == 6");
        py_run!(
            py,
            inst,
            "assert inst.get_pos_arg_kw_sep2(1, b=12, c=13) == 26"
        );
        py_expect_exception!(py, inst, "inst.get_pos_arg_kw_sep2(1, 2)", PyTypeError);

        py_run!(py, inst, "assert inst.get_pos_kw(1, b=2) == [1, {'b': 2}]");
        py_expect_exception!(py, inst, "inst.get_pos_kw(1,2)", PyTypeError);

        py_run!(py, inst, "assert inst.args_as_vec(1,2,3) == 6");
    });
}

#[pyclass]
/// A class with "documentation".
struct MethDocs {
    x: i32,
}

#[pymethods]
impl MethDocs {
    /// A method with "documentation" as well.
    fn method(&self) -> i32 {
        0
    }

    #[getter]
    /// `int`: a very "important" member of 'this' instance.
    fn get_x(&self) -> i32 {
        self.x
    }
}

#[test]
fn meth_doc() {
    Python::with_gil(|py| {
        let d = [("C", py.get_type_bound::<MethDocs>())].into_py_dict_bound(py);
        py_assert!(py, *d, "C.__doc__ == 'A class with \"documentation\".'");
        py_assert!(
            py,
            *d,
            "C.method.__doc__ == 'A method with \"documentation\" as well.'"
        );
        py_assert!(
            py,
            *d,
            "C.x.__doc__ == '`int`: a very \"important\" member of \\'this\\' instance.'"
        );
    });
}

#[pyclass]
struct MethodWithLifeTime {}

#[pymethods]
impl MethodWithLifeTime {
    fn set_to_list<'py>(&self, set: &Bound<'py, PySet>) -> PyResult<Bound<'py, PyList>> {
        let py = set.py();
        let mut items = vec![];
        for _ in 0..set.len() {
            items.push(set.pop().unwrap());
        }
        let list = PyList::new_bound(py, items);
        list.sort()?;
        Ok(list)
    }
}

#[test]
fn method_with_lifetime() {
    Python::with_gil(|py| {
        let obj = Py::new(py, MethodWithLifeTime {}).unwrap();
        py_run!(
            py,
            obj,
            "assert obj.set_to_list(set((1, 2, 3))) == [1, 2, 3]"
        );
    });
}

#[pyclass]
struct MethodWithPyClassArg {
    #[pyo3(get)]
    value: i64,
}

#[pymethods]
impl MethodWithPyClassArg {
    fn add(&self, other: &MethodWithPyClassArg) -> MethodWithPyClassArg {
        MethodWithPyClassArg {
            value: self.value + other.value,
        }
    }
    fn add_pyref(&self, other: PyRef<'_, MethodWithPyClassArg>) -> MethodWithPyClassArg {
        MethodWithPyClassArg {
            value: self.value + other.value,
        }
    }
    fn inplace_add(&self, other: &mut MethodWithPyClassArg) {
        other.value += self.value;
    }
    fn inplace_add_pyref(&self, mut other: PyRefMut<'_, MethodWithPyClassArg>) {
        other.value += self.value;
    }
    fn optional_add(&self, other: Option<&MethodWithPyClassArg>) -> MethodWithPyClassArg {
        MethodWithPyClassArg {
            value: self.value + other.map(|o| o.value).unwrap_or(10),
        }
    }
    fn optional_inplace_add(&self, other: Option<&mut MethodWithPyClassArg>) {
        if let Some(other) = other {
            other.value += self.value;
        }
    }
}

#[test]
fn method_with_pyclassarg() {
    Python::with_gil(|py| {
        let obj1 = Py::new(py, MethodWithPyClassArg { value: 10 }).unwrap();
        let obj2 = Py::new(py, MethodWithPyClassArg { value: 10 }).unwrap();
        let d = [("obj1", obj1), ("obj2", obj2)].into_py_dict_bound(py);
        py_run!(py, *d, "obj = obj1.add(obj2); assert obj.value == 20");
        py_run!(py, *d, "obj = obj1.add_pyref(obj2); assert obj.value == 20");
        py_run!(py, *d, "obj = obj1.optional_add(); assert obj.value == 20");
        py_run!(
            py,
            *d,
            "obj = obj1.optional_add(obj2); assert obj.value == 20"
        );
        py_run!(py, *d, "obj1.inplace_add(obj2); assert obj.value == 20");
        py_run!(
            py,
            *d,
            "obj1.inplace_add_pyref(obj2); assert obj2.value == 30"
        );
        py_run!(
            py,
            *d,
            "obj1.optional_inplace_add(); assert obj2.value == 30"
        );
        py_run!(
            py,
            *d,
            "obj1.optional_inplace_add(obj2); assert obj2.value == 40"
        );
    });
}

#[pyclass]
#[cfg(unix)]
struct CfgStruct {}

#[pyclass]
#[cfg(not(unix))]
struct CfgStruct {}

#[pymethods]
#[cfg(unix)]
impl CfgStruct {
    fn unix_method(&self) -> &str {
        "unix"
    }

    #[cfg(not(unix))]
    fn never_compiled_method(&self) {}
}

#[pymethods]
#[cfg(not(unix))]
impl CfgStruct {
    fn not_unix_method(&self) -> &str {
        "not unix"
    }

    #[cfg(unix)]
    fn never_compiled_method(&self) {}
}

#[test]
fn test_cfg_attrs() {
    Python::with_gil(|py| {
        let inst = Py::new(py, CfgStruct {}).unwrap();

        #[cfg(unix)]
        {
            py_assert!(py, inst, "inst.unix_method() == 'unix'");
            py_assert!(py, inst, "not hasattr(inst, 'not_unix_method')");
        }

        #[cfg(not(unix))]
        {
            py_assert!(py, inst, "not hasattr(inst, 'unix_method')");
            py_assert!(py, inst, "inst.not_unix_method() == 'not unix'");
        }

        py_assert!(py, inst, "not hasattr(inst, 'never_compiled_method')");
    });
}

#[pyclass]
#[derive(Default)]
struct FromSequence {
    #[pyo3(get)]
    numbers: Vec<i64>,
}

#[pymethods]
impl FromSequence {
    #[new]
    fn new(seq: Option<&pyo3::types::PySequence>) -> PyResult<Self> {
        if let Some(seq) = seq {
            Ok(FromSequence {
                numbers: seq.as_ref().extract::<Vec<_>>()?,
            })
        } else {
            Ok(FromSequence::default())
        }
    }
}

#[test]
fn test_from_sequence() {
    Python::with_gil(|py| {
        let typeobj = py.get_type_bound::<FromSequence>();
        py_assert!(py, typeobj, "typeobj(range(0, 4)).numbers == [0, 1, 2, 3]");
    });
}

#[pyclass]
struct r#RawIdents {
    #[pyo3(get, set)]
    r#type: PyObject,
    r#subtype: PyObject,
    r#subsubtype: PyObject,
}

#[pymethods]
impl r#RawIdents {
    #[new]
    pub fn r#new(
        r#_py: Python<'_>,
        r#type: PyObject,
        r#subtype: PyObject,
        r#subsubtype: PyObject,
    ) -> Self {
        Self {
            r#type,
            r#subtype,
            r#subsubtype,
        }
    }

    #[getter(r#subtype)]
    pub fn r#get_subtype(&self) -> PyObject {
        self.r#subtype.clone()
    }

    #[setter(r#subtype)]
    pub fn r#set_subtype(&mut self, r#subtype: PyObject) {
        self.r#subtype = r#subtype;
    }

    #[getter]
    pub fn r#get_subsubtype(&self) -> PyObject {
        self.r#subsubtype.clone()
    }

    #[setter]
    pub fn r#set_subsubtype(&mut self, r#subsubtype: PyObject) {
        self.r#subsubtype = r#subsubtype;
    }

    pub fn r#__call__(&mut self, r#type: PyObject) {
        self.r#type = r#type;
    }

    #[staticmethod]
    pub fn r#static_method(r#type: PyObject) -> PyObject {
        r#type
    }

    #[classmethod]
    pub fn r#class_method(_: &Bound<'_, PyType>, r#type: PyObject) -> PyObject {
        r#type
    }

    #[classattr]
    pub fn r#class_attr_fn() -> i32 {
        5
    }

    #[classattr]
    const r#CLASS_ATTR_CONST: i32 = 6;

    #[pyo3(signature = (r#struct = "foo"))]
    fn method_with_keyword<'a>(&self, r#struct: &'a str) -> &'a str {
        r#struct
    }
}

#[test]
fn test_raw_idents() {
    Python::with_gil(|py| {
        let raw_idents_type = py.get_type_bound::<r#RawIdents>();
        assert_eq!(raw_idents_type.qualname().unwrap(), "RawIdents");
        py_run!(
            py,
            raw_idents_type,
            r#"
            instance = raw_idents_type(type=None, subtype=5, subsubtype="foo")

            assert instance.type is None
            assert instance.subtype == 5
            assert instance.subsubtype == "foo"

            instance.type = 1
            instance.subtype = 2
            instance.subsubtype = 3

            assert instance.type == 1
            assert instance.subtype == 2
            assert instance.subsubtype == 3

            assert raw_idents_type.static_method(type=30) == 30
            assert instance.class_method(type=40) == 40

            instance(type=50)
            assert instance.type == 50

            assert raw_idents_type.class_attr_fn == 5
            assert raw_idents_type.CLASS_ATTR_CONST == 6

            assert instance.method_with_keyword() == "foo"
            assert instance.method_with_keyword("bar") == "bar"
            assert instance.method_with_keyword(struct="baz") == "baz"
            "#
        );
    })
}

// Regression test for issue 1505 - Python argument not detected correctly when inside a macro.

#[pyclass]
struct Issue1505 {}

macro_rules! pymethods {
    (
        #[pymethods]
        impl $ty: ty {
            fn $fn:ident (&self, $arg:ident : $arg_ty:ty) {}
        }
    ) => {
        #[pymethods]
        impl $ty {
            fn $fn(&self, $arg: $arg_ty) {}
        }
    };
}

pymethods!(
    #[pymethods]
    impl Issue1505 {
        fn issue_1505(&self, _py: Python<'_>) {}
    }
);

// Regression test for issue 1506 - incorrect macro hygiene.
// By applying the `#[pymethods]` attribute inside a macro_rules! macro, this separates the macro
// call scope from the scope of the impl block. For this to work our macros must be careful to not
// cheat hygiene!

#[pyclass]
struct Issue1506 {}

macro_rules! issue_1506 {
    (#[pymethods] $($body:tt)*) => {
        #[pymethods]
        $($body)*
    };
}

issue_1506!(
    #[pymethods]
    impl Issue1506 {
        fn issue_1506(
            &self,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        fn issue_1506_mut(
            &mut self,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        fn issue_1506_custom_receiver(
            _slf: Py<Self>,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        fn issue_1506_custom_receiver_explicit(
            _slf: Py<Issue1506>,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[new]
        fn issue_1506_new(
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) -> Self {
            Issue1506 {}
        }

        #[getter("foo")]
        fn issue_1506_getter(&self, _py: Python<'_>) -> i32 {
            5
        }

        #[setter("foo")]
        fn issue_1506_setter(&self, _py: Python<'_>, _value: i32) {}

        #[staticmethod]
        fn issue_1506_static(
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[classmethod]
        fn issue_1506_class(
            _cls: &Bound<'_, PyType>,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }
    }
);

#[pyclass]
struct Issue1696 {}

pymethods!(
    #[pymethods]
    impl Issue1696 {
        fn issue_1696(&self, _x: &InstanceMethod) {}
    }
);

#[test]
fn test_option_pyclass_arg() {
    // Option<&PyClass> argument with a default set in a signature regressed to a compile
    // error in PyO3 0.17.0 - this test it continues to be accepted.

    #[pyclass]
    struct SomePyClass {}

    #[pyfunction(signature = (arg=None))]
    fn option_class_arg(arg: Option<&SomePyClass>) -> Option<SomePyClass> {
        arg.map(|_| SomePyClass {})
    }

    Python::with_gil(|py| {
        let f = wrap_pyfunction_bound!(option_class_arg, py).unwrap();
        assert!(f.call0().unwrap().is_none());
        let obj = Py::new(py, SomePyClass {}).unwrap();
        assert!(f
            .call1((obj,))
            .unwrap()
            .extract::<Py<SomePyClass>>()
            .is_ok());
    })
}

#[test]
fn test_issue_2988() {
    #[pyfunction]
    #[pyo3(signature = (
        _data = vec![],
        _data2 = vec![],
    ))]
    pub fn _foo(
        _data: Vec<i32>,
        // The from_py_with here looks a little odd, we just need some way
        // to encourage the macro to expand the from_py_with default path too
        #[pyo3(from_py_with = "<Bound<'_, _> as PyAnyMethods>::extract")] _data2: Vec<i32>,
    ) {
    }
}
