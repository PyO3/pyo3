#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive1(i32); // newtype case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive2(i32, i32); // tuple case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
struct Derive3 {
    f: i32,
    #[pyo3(item(42))]
    g: i32,
    #[pyo3(default)]
    h: i32,
} // struct case

#[derive(crate::FromPyObject)]
#[pyo3(crate = "crate")]
enum Derive4 {
    A(i32),
    B { f: i32 },
} // enum case

crate::create_exception!(mymodule, CustomError, crate::exceptions::PyException);
crate::import_exception!(socket, gaierror);

fn intern(py: crate::Python<'_>) {
    let _foo = crate::intern!(py, "foo");
    let _bar = crate::intern!(py, stringify!(bar));
}

#[cfg(not(any(PyPy, GraalPy)))]
fn append_to_inittab() {
    #[crate::pymodule]
    #[pyo3(crate = "crate")]
    mod module_for_inittab {}

    crate::append_to_inittab!(module_for_inittab);
}

macro_rules! macro_rules_hygiene {
    ($name_a:ident, $name_b:ident) => {
        #[crate::pyclass(crate = "crate")]
        struct $name_a {}

        #[crate::pymethods(crate = "crate")]
        impl $name_a {
            fn finalize(&mut self) -> $name_b {
                $name_b {}
            }
        }

        #[crate::pyclass(crate = "crate")]
        struct $name_b {}
    };
}

macro_rules_hygiene!(MyClass1, MyClass2);

#[derive(crate::IntoPyObject, crate::IntoPyObjectRef)]
#[pyo3(crate = "crate")]
struct IntoPyObject1(i32); // transparent newtype case

#[derive(crate::IntoPyObject, crate::IntoPyObjectRef)]
#[pyo3(crate = "crate", transparent)]
struct IntoPyObject2<'a> {
    inner: &'a str, // transparent newtype case
}

#[derive(crate::IntoPyObject, crate::IntoPyObjectRef)]
#[pyo3(crate = "crate")]
struct IntoPyObject3<'py>(i32, crate::Bound<'py, crate::PyAny>); // tuple case

#[derive(crate::IntoPyObject, crate::IntoPyObjectRef)]
#[pyo3(crate = "crate")]
struct IntoPyObject4<'a, 'py> {
    callable: &'a crate::Bound<'py, crate::PyAny>, // struct case
    num: usize,
}

#[derive(crate::IntoPyObject, crate::IntoPyObjectRef)]
#[pyo3(crate = "crate")]
enum IntoPyObject5<'a, 'py> {
    TransparentTuple(i32),
    #[pyo3(transparent)]
    TransparentStruct {
        f: crate::Py<crate::PyAny>,
    },
    Tuple(crate::Bound<'py, crate::types::PyString>, usize),
    Struct {
        f: i32,
        g: &'a str,
    },
} // enum case
