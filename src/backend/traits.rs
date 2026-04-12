/// A backend-specific runtime realization.
pub trait Backend {
    /// The interpreter handle exposed by the backend.
    type Interpreter: BackendInterpreter;

    /// The class builder used to realize `#[pyclass]` declarations.
    type ClassBuilder<'py>: BackendClassBuilder<'py>
    where
        Self: 'py;

    /// The function builder used to realize `#[pyfunction]` and `#[pymethods]`.
    type FunctionBuilder<'py>: BackendFunctionBuilder<'py>
    where
        Self: 'py;
}

/// Marker trait for a backend interpreter handle.
pub trait BackendInterpreter {}

/// Marker trait for backend class builders.
pub trait BackendClassBuilder<'py> {}

/// Marker trait for backend function builders.
pub trait BackendFunctionBuilder<'py> {}

impl BackendInterpreter for () {}
impl<'py> BackendClassBuilder<'py> for () {}
impl<'py> BackendFunctionBuilder<'py> for () {}
