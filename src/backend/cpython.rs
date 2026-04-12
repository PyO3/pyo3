use super::traits::Backend;

/// CPython backend marker.
pub struct Cpython;

impl Backend for Cpython {
    type Interpreter = ();
    type ClassBuilder<'py> = ()
    where
        Self: 'py;
    type FunctionBuilder<'py> = ()
    where
        Self: 'py;
}
