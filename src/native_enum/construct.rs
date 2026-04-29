use crate::types::any::PyAnyMethods;
use crate::types::dict::PyDictMethods;
use crate::types::{PyAny, PyDict, PyList, PyTuple, PyType};
use crate::{Bound, IntoPyObject, PyResult, Python};

use super::base_cache::{get_cached_base, get_enum_auto};
use super::spec::{NativeEnumSpec, VariantValue};

/// Builds a Python `enum` subclass from `spec` without caching the result.
///
/// The base class (e.g. `enum.Enum`) is retrieved from a per-interpreter cache, so
/// the `enum` module is imported only once. The generated class itself is **not** cached;
/// each call returns a freshly constructed `Bound<'py, PyType>`.
pub fn build_native_enum<'py>(
    py: Python<'py>,
    spec: &NativeEnumSpec,
) -> PyResult<Bound<'py, PyType>> {
    let base_cls = get_cached_base(py, spec.base)?;

    let members: Vec<Bound<'py, PyTuple>> = spec
        .variants
        .iter()
        .map(|(variant_name, value)| {
            let py_value: Bound<'py, PyAny> = match value {
                VariantValue::Int(v) => v.into_pyobject(py)?.into_any(),
                VariantValue::Str(s) => s.into_pyobject(py)?.into_any(),
                VariantValue::Auto => get_enum_auto(py)?.call0()?,
            };
            let name_obj = variant_name.into_pyobject(py)?.into_any();
            PyTuple::new(py, [name_obj, py_value])
        })
        .collect::<PyResult<_>>()?;

    let members_list = PyList::new(py, members)?;
    let name_arg = spec.name.into_pyobject(py)?.into_any();
    let args = PyTuple::new(py, [name_arg, members_list.into_any()])?;

    let class = if spec.module.is_some() || spec.qualname.is_some() {
        let kwargs = PyDict::new(py);
        if let Some(m) = spec.module {
            kwargs.set_item("module", m)?;
        }
        if let Some(q) = spec.qualname {
            kwargs.set_item("qualname", q)?;
        }
        base_cls.call(args, Some(&kwargs))?
    } else {
        base_cls.call1(args)?
    };

    class.cast_into::<PyType>().map_err(Into::into)
}
