#![cfg(feature = "uuid")]
//! TODO
use uuid::Uuid;

use crate::exceptions::PyValueError;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::bytes::PyBytes;
use crate::types::dict::IntoPyDict;
use crate::types::{PyBytesMethods, PyType};
use crate::{
    intern, Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject,
};

impl FromPyObject<'_> for Uuid {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Not propagating Err up here because the UUID class should be an invariant.
        let uuid_cls = get_uuid_cls(ob.py()).expect("failed to load uuid.UUID");
        let py_bytes: Bound<'_, PyBytes> = if ob.is_exact_instance(&uuid_cls) {
            ob.getattr(intern!(ob.py(), "bytes"))?.downcast_into()?
        } else {
            ob.extract()?
        };

        Uuid::from_slice(py_bytes.as_bytes())
            .map_err(|_| PyValueError::new_err("The given value is not a valid UUID."))
    }
}

static UUID_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

#[inline(always)]
fn get_uuid_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    UUID_CLS.get_or_try_init_type_ref(py, "uuid", "UUID")
}

#[inline(always)]
fn into_uuid_from_pybytes(py: Python<'_>, py_bytes: Bound<'_, PyBytes>) -> PyObject {
    let uuid_cls = get_uuid_cls(py).expect("failed to load uuid.UUID");
    let kwargs = vec![(intern!(py, "bytes"), py_bytes)].into_py_dict_bound(py);

    uuid_cls
        .call((), Some(&kwargs))
        .expect("failed to call uuid.UUID")
        .to_object(py)
}

impl ToPyObject for Uuid {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let rs_bytes = self.as_bytes();
        let py_bytes = PyBytes::new_bound(py, rs_bytes);

        into_uuid_from_pybytes(py, py_bytes)
    }
}

impl IntoPy<PyObject> for Uuid {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let rs_bytes = self.into_bytes();
        let py_bytes =
            unsafe { PyBytes::bound_from_ptr(py, &rs_bytes as *const u8, rs_bytes.len()) };

        into_uuid_from_pybytes(py, py_bytes)
    }
}

#[cfg(test)]
mod test_uuid {
    use super::*;
    use uuid::Uuid;

    use crate::types::PyDict;
    use crate::Python;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() -> PyResult<()> {
                Python::with_gil(|py| {
                    let rs_orig = $rs;
                    let rs_uuid = rs_orig.into_py(py);
                    let locals = PyDict::new_bound(py);
                    locals.set_item("rs_uuid", &rs_uuid)?;

                    py.run_bound(
                        &format!(
                            "import uuid\npy_uuid = uuid.UUID('{}')\nassert py_uuid == rs_uuid",
                            $py
                        ),
                        None,
                        Some(&locals),
                    )?;

                    let py_uuid = locals.get_item("py_uuid")?;
                    let py_result: Uuid = py_uuid.extract()?;
                    assert_eq!(rs_orig, py_result);

                    Ok(())
                })
            }
        };
    }

    convert_constants!(
        convert_nil,
        Uuid::nil(),
        "00000000-0000-0000-0000-000000000000"
    );
    convert_constants!(
        convert_max,
        Uuid::max(),
        "ffffffff-ffff-ffff-ffff-ffffffffffff"
    );
    convert_constants!(
        convert_random_v4,
        Uuid::parse_str("a4f6d1b9-1898-418f-b11d-ecc6fe1e1f00").unwrap(),
        "a4f6d1b9-1898-418f-b11d-ecc6fe1e1f00"
    );
}
