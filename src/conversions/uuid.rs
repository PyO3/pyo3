#![cfg(feature = "uuid")]
//! TODO
use uuid::Uuid;

use crate::exceptions::PyValueError;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::bytes::PyBytes;
use crate::types::dict::IntoPyDict;
use crate::types::string::PyStringMethods;
use crate::types::PyType;
use crate::{Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};

impl FromPyObject<'_> for Uuid {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_bytes: PyResult<Vec<u8>> = ob.extract();

        match py_bytes {
            Ok(val) => Uuid::try_parse_ascii(&val)
                .map_err(|_| PyValueError::new_err("The given value is not a valid UUID.")),
            Err(_) => {
                let py_str = ob.str()?;
                let rs_str = py_str.to_cow()?;

                Uuid::try_parse(&rs_str)
                    .map_err(|_| PyValueError::new_err("The given value is not a valid UUID."))
            }
        }
    }
}

static UUID_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_uuid_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    UUID_CLS.get_or_try_init_type_ref(py, "uuid", "UUID")
}

impl ToPyObject for Uuid {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        let uuid_cls = get_uuid_cls(py).expect("failed to load uuid.UUID");

        let rs_bytes = self.as_bytes();
        let py_bytes = PyBytes::new_bound(py, rs_bytes);

        let kwargs = vec![("bytes", py_bytes)].into_py_dict_bound(py);
        uuid_cls
            .call((), Some(&kwargs))
            .expect("failed to call uuid.UUID")
            .to_object(py)
    }
}

impl IntoPy<PyObject> for Uuid {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
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
}
