use core::marker::PhantomData;

use crate::backend::{
    spec::BackendKind,
    traits::{Backend, BackendClassBuilder, BackendFunctionBuilder, BackendInterpreter},
};

/// RustPython backend marker.
pub struct RustPythonBackend;

impl Backend for RustPythonBackend {
    const KIND: BackendKind = BackendKind::Rustpython;

    type Interpreter = RustPythonInterpreter;
    type ClassBuilder<'py>
        = RustPythonClassBuilder<'py>
    where
        Self: 'py;
    type FunctionBuilder<'py>
        = RustPythonFunctionBuilder<'py>
    where
        Self: 'py;
}

/// Placeholder RustPython interpreter handle.
pub struct RustPythonInterpreter;

impl BackendInterpreter for RustPythonInterpreter {}

/// Placeholder RustPython class builder.
pub struct RustPythonClassBuilder<'py> {
    _phantom: PhantomData<&'py ()>,
}

impl<'py> BackendClassBuilder<'py> for RustPythonClassBuilder<'py> {}

/// Placeholder RustPython function builder.
pub struct RustPythonFunctionBuilder<'py> {
    _phantom: PhantomData<&'py ()>,
}

impl<'py> BackendFunctionBuilder<'py> for RustPythonFunctionBuilder<'py> {}
