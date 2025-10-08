#![cfg(feature = "macros")]

#[cfg(not(Py_LIMITED_API))]
use pyo3::exceptions::PyWarning;
use pyo3::exceptions::{PyFutureWarning, PyUserWarning};
use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::PySequence;
use pyo3::types::{IntoPyDict, PyDict, PyList, PySet, PyString, PyTuple, PyType};
use pyo3::BoundObject;
use pyo3_macros::pyclass;

use crate::test_utils::CatchWarnings;

mod test_utils;

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
    Python::attach(|py| {
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
    Python::attach(|py| {
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
    fn method_owned(cls: Py<PyType>, py: Python<'_>) -> PyResult<String> {
        Ok(format!("{}.method_owned()!", cls.bind(py).qualname()?))
    }
}

#[test]
fn class_method() {
    Python::attach(|py| {
        let d = [("C", py.get_type::<ClassMethod>())]
            .into_py_dict(py)
            .unwrap();
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
    Python::attach(|py| {
        let d = [("C", py.get_type::<ClassMethodWithArgs>())]
            .into_py_dict(py)
            .unwrap();
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
    Python::attach(|py| {
        assert_eq!(StaticMethod::method(py), "StaticMethod.method()!");

        let d = [("C", py.get_type::<StaticMethod>())]
            .into_py_dict(py)
            .unwrap();
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
        format!("0x{input:x}")
    }
}

#[test]
fn static_method_with_args() {
    Python::attach(|py| {
        assert_eq!(StaticMethodWithArgs::method(py, 1234), "0x4d2");

        let d = [("C", py.get_type::<StaticMethodWithArgs>())]
            .into_py_dict(py)
            .unwrap();
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
    #[pyo3(signature=(_t1 = None, t2 = None, _t3 = None))]
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
    fn get_kwargs<'py>(
        &self,
        py: Python<'py>,
        args: &Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        [
            args.as_any().clone(),
            kwargs.into_pyobject(py)?.into_any().into_bound(),
        ]
        .into_pyobject(py)
    }

    #[pyo3(signature = (a, *args, **kwargs))]
    fn get_pos_arg_kw<'py>(
        &self,
        py: Python<'py>,
        a: i32,
        args: &Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        [
            a.into_pyobject(py)?.into_any().into_bound(),
            args.as_any().clone(),
            kwargs.into_pyobject(py)?.into_any().into_bound(),
        ]
        .into_pyobject(py)
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
    ) -> PyResult<Py<PyAny>> {
        [
            a.into_pyobject(py)?.into_any().into_bound(),
            kwargs.into_pyobject(py)?.into_any().into_bound(),
        ]
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (a=0, /, **kwargs))]
    fn get_optional_pos_only_with_kwargs(
        &self,
        py: Python<'_>,
        a: i32,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        [
            a.into_pyobject(py)?.into_any().into_bound(),
            kwargs.into_pyobject(py)?.into_any().into_bound(),
        ]
        .into_pyobject(py)
        .map(Bound::unbind)
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
    ) -> PyResult<Py<PyAny>> {
        (args, a)
            .into_pyobject(py)
            .map(BoundObject::into_any)
            .map(Bound::unbind)
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
    fn get_pos_kw(
        &self,
        py: Python<'_>,
        a: i32,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        [
            a.into_pyobject(py)?.into_any().into_bound(),
            kwargs.into_pyobject(py)?.into_any().into_bound(),
        ]
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    // "args" can be anything that can be extracted from PyTuple
    #[pyo3(signature = (*args))]
    fn args_as_vec(&self, args: Vec<i32>) -> i32 {
        args.iter().sum()
    }
}

#[test]
fn meth_signature() {
    Python::attach(|py| {
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
    Python::attach(|py| {
        let d = [("C", py.get_type::<MethDocs>())].into_py_dict(py).unwrap();
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
        let list = PyList::new(py, items)?;
        list.sort()?;
        Ok(list)
    }
}

#[test]
fn method_with_lifetime() {
    Python::attach(|py| {
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
    #[pyo3(signature=(other = None))]
    fn optional_add(&self, other: Option<&MethodWithPyClassArg>) -> MethodWithPyClassArg {
        MethodWithPyClassArg {
            value: self.value + other.map(|o| o.value).unwrap_or(10),
        }
    }
    #[pyo3(signature=(other = None))]
    fn optional_inplace_add(&self, other: Option<&mut MethodWithPyClassArg>) {
        if let Some(other) = other {
            other.value += self.value;
        }
    }
}

#[test]
fn method_with_pyclassarg() {
    Python::attach(|py| {
        let obj1 = Py::new(py, MethodWithPyClassArg { value: 10 }).unwrap();
        let obj2 = Py::new(py, MethodWithPyClassArg { value: 10 }).unwrap();
        let d = [("obj1", obj1), ("obj2", obj2)].into_py_dict(py).unwrap();
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
    Python::attach(|py| {
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
    #[pyo3(signature=(seq = None))]
    fn new(seq: Option<&Bound<'_, PySequence>>) -> PyResult<Self> {
        if let Some(seq) = seq {
            Ok(FromSequence {
                numbers: seq.as_any().extract::<Vec<_>>()?,
            })
        } else {
            Ok(FromSequence::default())
        }
    }
}

#[test]
fn test_from_sequence() {
    Python::attach(|py| {
        let typeobj = py.get_type::<FromSequence>();
        py_assert!(py, typeobj, "typeobj(range(0, 4)).numbers == [0, 1, 2, 3]");
    });
}

#[pyclass]
struct r#RawIdents {
    #[pyo3(get, set)]
    r#type: Py<PyAny>,
    r#subtype: Py<PyAny>,
    r#subsubtype: Py<PyAny>,
}

#[pymethods]
impl r#RawIdents {
    #[new]
    pub fn r#new(
        r#_py: Python<'_>,
        r#type: Py<PyAny>,
        r#subtype: Py<PyAny>,
        r#subsubtype: Py<PyAny>,
    ) -> Self {
        Self {
            r#type,
            r#subtype,
            r#subsubtype,
        }
    }

    #[getter(r#subtype)]
    pub fn r#get_subtype(&self, py: Python<'_>) -> Py<PyAny> {
        self.r#subtype.clone_ref(py)
    }

    #[setter(r#subtype)]
    pub fn r#set_subtype(&mut self, r#subtype: Py<PyAny>) {
        self.r#subtype = r#subtype;
    }

    #[getter]
    pub fn r#get_subsubtype(&self, py: Python<'_>) -> Py<PyAny> {
        self.r#subsubtype.clone_ref(py)
    }

    #[setter]
    pub fn r#set_subsubtype(&mut self, r#subsubtype: Py<PyAny>) {
        self.r#subsubtype = r#subsubtype;
    }

    pub fn r#__call__(&mut self, r#type: Py<PyAny>) {
        self.r#type = r#type;
    }

    #[staticmethod]
    pub fn r#static_method(r#type: Py<PyAny>) -> Py<PyAny> {
        r#type
    }

    #[classmethod]
    pub fn r#class_method(_: &Bound<'_, PyType>, r#type: Py<PyAny>) -> Py<PyAny> {
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
    Python::attach(|py| {
        let raw_idents_type = py.get_type::<r#RawIdents>();
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
        #[pyo3(signature = (_arg, _args, _kwargs=None))]
        fn issue_1506(
            &self,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[pyo3(signature = (_arg, _args, _kwargs=None))]
        fn issue_1506_mut(
            &mut self,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[pyo3(signature = (_arg, _args, _kwargs=None))]
        fn issue_1506_custom_receiver(
            _slf: Py<Self>,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[pyo3(signature = (_arg, _args, _kwargs=None))]
        fn issue_1506_custom_receiver_explicit(
            _slf: Py<Issue1506>,
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[new]
        #[pyo3(signature = (_arg, _args, _kwargs=None))]
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
        #[pyo3(signature = (_arg, _args, _kwargs=None))]
        fn issue_1506_static(
            _py: Python<'_>,
            _arg: &Bound<'_, PyAny>,
            _args: &Bound<'_, PyTuple>,
            _kwargs: Option<&Bound<'_, PyDict>>,
        ) {
        }

        #[classmethod]
        #[pyo3(signature = (_arg, _args, _kwargs=None))]
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

    Python::attach(|py| {
        let f = wrap_pyfunction!(option_class_arg, py).unwrap();
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
        #[pyo3(from_py_with = <Bound<'_, _> as PyAnyMethods>::extract)] _data2: Vec<i32>,
    ) {
    }
}

#[cfg(not(Py_LIMITED_API))]
#[pyclass(extends=PyWarning)]
pub struct UserDefinedWarning {}

#[cfg(not(Py_LIMITED_API))]
#[pymethods]
impl UserDefinedWarning {
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: Bound<'_, PyAny>, _kwargs: Option<Bound<'_, PyAny>>) -> Self {
        Self {}
    }
}

#[test]
fn test_pymethods_warn() {
    // We do not test #[classattr] nor __traverse__
    // because it doesn't make sense to implement deprecated methods for them.

    #[pyclass]
    struct WarningMethodContainer {
        value: i32,
    }

    #[pymethods]
    impl WarningMethodContainer {
        #[new]
        #[pyo3(warn(message = "this __new__ method raises warning"))]
        fn new() -> Self {
            Self { value: 0 }
        }

        #[pyo3(warn(message = "this method raises warning"))]
        fn method_with_warning(_slf: PyRef<'_, Self>) {}

        #[pyo3(warn(message = "this method raises warning", category = PyFutureWarning))]
        fn method_with_warning_and_custom_category(_slf: PyRef<'_, Self>) {}

        #[cfg(not(Py_LIMITED_API))]
        #[pyo3(warn(message = "this method raises user-defined warning", category = UserDefinedWarning))]
        fn method_with_warning_and_user_defined_category(&self) {}

        #[staticmethod]
        #[pyo3(warn(message = "this static method raises warning"))]
        fn static_method() {}

        #[staticmethod]
        #[pyo3(warn(message = "this class method raises warning"))]
        fn class_method() {}

        #[getter]
        #[pyo3(warn(message = "this getter raises warning"))]
        fn get_value(&self) -> i32 {
            self.value
        }

        #[setter]
        #[pyo3(warn(message = "this setter raises warning"))]
        fn set_value(&mut self, value: i32) {
            self.value = value;
        }

        #[pyo3(warn(message = "this subscript op method raises warning"))]
        fn __getitem__(&self, _key: i32) -> i32 {
            0
        }

        #[pyo3(warn(message = "the + op method raises warning"))]
        fn __add__(&self, other: PyRef<'_, Self>) -> Self {
            Self {
                value: self.value + other.value,
            }
        }

        #[pyo3(warn(message = "this __call__ method raises warning"))]
        fn __call__(&self) -> i32 {
            self.value
        }
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<WarningMethodContainer>();
        let obj = CatchWarnings::enter(py, |_| typeobj.call0()).unwrap();

        // FnType::Fn
        py_expect_warning!(
            py,
            obj,
            "obj.method_with_warning()",
            [("this method raises warning", PyUserWarning)],
        );

        // FnType::Fn
        py_expect_warning!(
            py,
            obj,
            "obj.method_with_warning_and_custom_category()",
            [("this method raises warning", PyFutureWarning)]
        );

        // FnType::Fn, user-defined warning
        #[cfg(not(Py_LIMITED_API))]
        py_expect_warning!(
            py,
            obj,
            "obj.method_with_warning_and_user_defined_category()",
            [(
                "this method raises user-defined warning",
                UserDefinedWarning
            )]
        );

        // #[staticmethod], FnType::FnStatic
        py_expect_warning!(
            py,
            typeobj,
            "typeobj.static_method()",
            [("this static method raises warning", PyUserWarning)]
        );

        // #[classmethod], FnType::FnClass
        py_expect_warning!(
            py,
            typeobj,
            "typeobj.class_method()",
            [("this class method raises warning", PyUserWarning)]
        );

        // #[classmethod], FnType::FnClass
        py_expect_warning!(
            py,
            obj,
            "obj.class_method()",
            [("this class method raises warning", PyUserWarning)]
        );

        // #[new], FnType::FnNew
        py_expect_warning!(
            py,
            typeobj,
            "typeobj()",
            [("this __new__ method raises warning", PyUserWarning)]
        );

        // #[getter], FnType::Getter
        py_expect_warning!(
            py,
            obj,
            "val = obj.value",
            [("this getter raises warning", PyUserWarning)]
        );

        // #[setter], FnType::Setter
        py_expect_warning!(
            py,
            obj,
            "obj.value = 10",
            [("this setter raises warning", PyUserWarning)]
        );

        // PyMethodProtoKind::Slot
        py_expect_warning!(
            py,
            obj,
            "obj[0]",
            [("this subscript op method raises warning", PyUserWarning)]
        );

        // PyMethodProtoKind::SlotFragment
        py_expect_warning!(
            py,
            obj,
            "obj + obj",
            [("the + op method raises warning", PyUserWarning)]
        );

        // PyMethodProtoKind::Call
        py_expect_warning!(
            py,
            obj,
            "obj()",
            [("this __call__ method raises warning", PyUserWarning)]
        );
    });

    #[pyclass]
    struct WarningMethodContainer2 {}

    #[pymethods]
    impl WarningMethodContainer2 {
        #[new]
        #[classmethod]
        #[pyo3(warn(message = "this class-method __new__ method raises warning"))]
        fn new(_cls: Bound<'_, PyType>) -> Self {
            Self {}
        }
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<WarningMethodContainer2>();

        // #[new], #[classmethod], FnType::FnNewClass
        py_expect_warning!(
            py,
            typeobj,
            "typeobj()",
            [(
                "this class-method __new__ method raises warning",
                PyUserWarning
            )]
        );
    });
}

#[test]
fn test_py_methods_multiple_warn() {
    #[pyclass]
    struct MultipleWarnContainer {}

    #[pymethods]
    impl MultipleWarnContainer {
        #[new]
        fn new() -> Self {
            Self {}
        }

        #[pyo3(warn(message = "this method raises warning 1"))]
        #[pyo3(warn(message = "this method raises warning 2", category = PyFutureWarning))]
        fn multiple_warn_method(&self) {}

        #[cfg(not(Py_LIMITED_API))]
        #[pyo3(warn(message = "this method raises FutureWarning", category = PyFutureWarning))]
        #[pyo3(warn(message = "this method raises UserDefinedWarning", category = UserDefinedWarning))]
        fn multiple_warn_custom_category_method(&self) {}
    }

    Python::attach(|py| {
        let typeobj = py.get_type::<MultipleWarnContainer>();
        let obj = typeobj.call0().unwrap();

        py_expect_warning!(
            py,
            obj,
            "obj.multiple_warn_method()",
            [
                ("this method raises warning 1", PyUserWarning),
                ("this method raises warning 2", PyFutureWarning)
            ]
        );

        #[cfg(not(Py_LIMITED_API))]
        py_expect_warning!(
            py,
            obj,
            "obj.multiple_warn_custom_category_method()",
            [
                ("this method raises FutureWarning", PyFutureWarning),
                ("this method raises UserDefinedWarning", UserDefinedWarning)
            ]
        );
    });
}
