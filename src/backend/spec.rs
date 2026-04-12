/// Identifies which backend a lowered semantic spec targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendKind {
    /// CPython is the reference backend.
    Cpython,
    /// RustPython is the motivating backend for the split.
    Rustpython,
}

/// Minimal backend-neutral lowering payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendSpec {
    /// Target backend selection.
    pub kind: BackendKind,
}

impl BackendSpec {
    /// Creates a new backend spec.
    pub const fn new(kind: BackendKind) -> Self {
        Self { kind }
    }
}
