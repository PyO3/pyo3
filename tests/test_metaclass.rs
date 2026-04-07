#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::{PyAny, PyDict, PyString, PyTuple, PyType};

mod test_utils;

// A simple metaclass with no extra data fields or custom __new__.
// It relies on the inherited `type.__new__` for class creation.
// `subclass` is set so this metaclass can itself be used as a base (e.g. for DerivedMeta).
#[pyclass(extends = PyType, subclass)]
struct SimpleMeta;

#[pymethods]
impl SimpleMeta {
    // Overrides isinstance(x, C) where C has metaclass SimpleMeta.
    // Always returns True for testing purposes.
    fn __instancecheck__(&self, _instance: &Bound<'_, PyAny>) -> bool {
        true
    }

    // Overrides issubclass(X, C) where C has metaclass SimpleMeta.
    // Always returns True for testing purposes.
    fn __subclasscheck__(&self, _subclass: &Bound<'_, PyAny>) -> bool {
        true
    }

    // Overrides C[item] where C has metaclass SimpleMeta.
    fn __getitem__(&self, item: Py<PyAny>) -> Py<PyAny> {
        item
    }
}

// A metaclass that demonstrates __prepare__ (returns a custom namespace dict).
#[pyclass(extends = PyType)]
struct PrepMeta;

#[pymethods]
impl PrepMeta {
    // __prepare__ is a classmethod called before the class body is executed.
    // It should return a mapping (typically a dict) used as the class namespace.
    #[classmethod]
    #[pyo3(signature = (_name, _bases, **_kwargs))]
    fn __prepare__(
        _mcs: &Bound<'_, PyType>,
        _name: &str,
        _bases: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyDict>> {
        let py = _mcs.py();
        let d = PyDict::new(py);
        d.set_item("__prepared__", true)?;
        Ok(d.into())
    }
}

// A metaclass that overrides __call__ so that calling C() returns a tuple
// (cls, args, kwargs) for testing.
#[pyclass(extends = PyType)]
struct CallMeta;

#[pymethods]
impl CallMeta {
    #[pyo3(signature = (*args, **_kwargs))]
    fn __call__(
        slf: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        // Return (type, args) to show we intercept the call
        let result = PyTuple::new(py, [slf.as_any().clone(), args.as_any().clone()])?;
        Ok(result.into())
    }
}

// A metaclass with a custom __new__ that uses the safe PyType::metaclass_type_new helper.
#[pyclass(extends = PyType)]
struct CustomNewMeta;

#[pymethods]
impl CustomNewMeta {
    #[new]
    #[classmethod]
    fn new(
        cls: &Bound<'_, PyType>,
        name: &Bound<'_, PyString>,
        bases: &Bound<'_, PyTuple>,
        namespace: &Bound<'_, PyDict>,
    ) -> PyResult<Py<Self>> {
        // Use the safe helper instead of raw FFI.
        PyType::metaclass_type_new(cls, name, bases, namespace)?
            .cast_into::<Self>()
            .map(|b| b.unbind())
            .map_err(Into::into)
    }
}

// A Rust metaclass that extends another Rust metaclass.
// Uses #[pyclass(extends = SimpleMeta)] to inherit from SimpleMeta
// (which must have `subclass` to be usable as a base).
#[pyclass(extends = SimpleMeta)]
struct DerivedMeta;

#[pymethods]
impl DerivedMeta {
    #[new]
    #[classmethod]
    fn new(
        cls: &Bound<'_, PyType>,
        name: &Bound<'_, PyString>,
        bases: &Bound<'_, PyTuple>,
        namespace: &Bound<'_, PyDict>,
    ) -> PyResult<Py<Self>> {
        SimpleMeta::new(cls, &(name.to_string() + "-derived"), bases, namespace)?
            .cast_into::<Self>()
            .map(|b| b.unbind())
            .map_err(Into::into)
    }
}

#[test]
fn test_simple_metaclass_type_hierarchy() {
    Python::attach(|py| {
        let meta = py.get_type::<SimpleMeta>();
        py_run!(
            py,
            meta,
            r#"
# SimpleMeta must be a subclass of type
assert issubclass(meta, type), f"Expected issubclass(meta, type) but got False"
# A class created with metaclass=meta must be an instance of meta
class C(metaclass=meta): pass
assert isinstance(C, meta), f"Expected isinstance(C, meta) but got False"
assert type(C) is meta, f"Expected type(C) is meta but got {type(C)}"
"#
        );
    });
}

#[test]
fn test_metaclass_instancecheck() {
    Python::attach(|py| {
        let meta = py.get_type::<SimpleMeta>();
        py_run!(
            py,
            meta,
            r#"
class C(metaclass=meta): pass
# __instancecheck__ on SimpleMeta always returns True
assert isinstance(42, C)
assert isinstance("hello", C)
assert isinstance(None, C)
"#
        );
    });
}

#[test]
fn test_metaclass_subclasscheck() {
    Python::attach(|py| {
        let meta = py.get_type::<SimpleMeta>();
        py_run!(
            py,
            meta,
            r#"
class C(metaclass=meta): pass
# __subclasscheck__ on SimpleMeta always returns True
assert issubclass(int, C)
assert issubclass(str, C)
assert issubclass(list, C)
"#
        );
    });
}

#[test]
fn test_metaclass_getitem() {
    Python::attach(|py| {
        let meta = py.get_type::<SimpleMeta>();
        py_run!(
            py,
            meta,
            r#"
class C(metaclass=meta): pass
# __getitem__ on SimpleMeta returns the item unchanged
assert C[int] is int
assert C[str] is str
# Tuple subscript creates a tuple
tup = C[int, str]
assert tup == (int, str)
"#
        );
    });
}

#[test]
fn test_metaclass_prepare() {
    Python::attach(|py| {
        let meta = py.get_type::<PrepMeta>();
        py_run!(
            py,
            meta,
            r#"
# __prepare__ injects '__prepared__' into the class namespace before body runs
class C(metaclass=meta): pass
assert C.__prepared__ is True, f"Expected C.__prepared__ == True, got {C.__dict__.get('__prepared__')}"
"#
        );
    });
}

#[test]
fn test_metaclass_call() {
    Python::attach(|py| {
        let meta = py.get_type::<CallMeta>();
        py_run!(
            py,
            meta,
            r#"
class D(metaclass=meta): pass
# Calling D() invokes CallMeta.__call__(D, (), {})
result = D()
assert isinstance(result, tuple)
assert result[0] is D
"#
        );
    });
}

#[test]
fn test_metaclass_custom_new() {
    Python::attach(|py| {
        let meta = py.get_type::<CustomNewMeta>();
        py_run!(
            py,
            meta,
            r#"
class E(metaclass=meta): pass
assert isinstance(E, meta), f"Expected isinstance(E, meta) but got False"
assert issubclass(meta, type), f"Expected issubclass(meta, type) but got False"
"#
        );
    });
}

#[test]
fn test_metaclass_is_subclassable() {
    Python::attach(|py| {
        let meta = py.get_type::<SimpleMeta>();
        // SimpleMeta should be subclassable (i.e., can be used as a metaclass)
        py_run!(
            py,
            meta,
            r#"
# Metaclasses can be subclassed in Python
class SubMeta(meta): pass
class F(metaclass=SubMeta): pass
assert isinstance(F, SubMeta)
assert isinstance(F, meta)
"#
        );
    });
}

#[test]
fn test_rust_metaclass_extends_rust_metaclass() {
    Python::attach(|py| {
        let base_meta = py.get_type::<SimpleMeta>();
        let derived_meta = py.get_type::<DerivedMeta>();
        py_run!(
            py,
            base_meta derived_meta,
            r#"
# DerivedMeta must be a subclass of SimpleMeta (and hence of type)
assert issubclass(derived_meta, base_meta), f"Expected issubclass(derived_meta, base_meta)"
assert issubclass(derived_meta, type), f"Expected issubclass(derived_meta, type)"
# Classes using DerivedMeta must be instances of both
class G(metaclass=derived_meta): pass
assert isinstance(G, derived_meta)
assert isinstance(G, base_meta)
assert issubclass(type(G), derived_meta)
assert G.__name__ == "G-derived", f"Expected G.__name__ == 'G-derived', got {G.__name__}"
"#
        );
    });
}
