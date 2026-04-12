use super::traits::Backend;

/// RustPython backend marker.
pub struct Rustpython;

impl Backend for Rustpython {
    type Interpreter = ();
    type ClassBuilder<'py> = ()
    where
        Self: 'py;
    type FunctionBuilder<'py> = ()
    where
        Self: 'py;
}
