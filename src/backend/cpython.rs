use core::marker::PhantomData;

use crate::backend::{
    spec::BackendKind,
    traits::{Backend, BackendClassBuilder, BackendFunctionBuilder, BackendInterpreter},
};

/// Reference CPython-family backend marker.
pub struct CpythonBackend;

impl Backend for CpythonBackend {
    const KIND: BackendKind = BackendKind::Cpython;

    type Interpreter = CpythonInterpreter;
    type ClassBuilder<'py>
        = CpythonClassBuilder<'py>
    where
        Self: 'py;
    type FunctionBuilder<'py>
        = CpythonFunctionBuilder<'py>
    where
        Self: 'py;
}

/// Placeholder CPython-family interpreter handle.
pub struct CpythonInterpreter;

impl BackendInterpreter for CpythonInterpreter {}

/// Placeholder CPython-family class builder.
pub struct CpythonClassBuilder<'py> {
    _phantom: PhantomData<&'py ()>,
}

impl<'py> BackendClassBuilder<'py> for CpythonClassBuilder<'py> {}

/// Placeholder CPython-family function builder.
pub struct CpythonFunctionBuilder<'py> {
    _phantom: PhantomData<&'py ()>,
}

impl<'py> BackendFunctionBuilder<'py> for CpythonFunctionBuilder<'py> {}
